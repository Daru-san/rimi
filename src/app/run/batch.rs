use std::mem::take;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::app::command::{ImageArgs, ImageCommand};
use crate::backend::error::TaskError;
use crate::backend::paths::{create_paths, paths_exist, prompt_overwrite};
use crate::backend::progress::{AppProgress, BatchProgress};
use crate::backend::queue::{TaskQueue, TaskState};
use crate::image::manipulator::{open_image, save_image_format};

use super::RunBatch;
use anyhow::Result;
use rayon::ThreadPoolBuilder;

const TASK_COUNT: usize = 3;

struct BatchRunner {
    tasks_queue: Arc<Mutex<TaskQueue>>,
    progress: Arc<Mutex<BatchProgress>>,
}

impl BatchRunner {
    fn init(verbosity: u32) -> Self {
        Self {
            tasks_queue: Arc::new(Mutex::new(TaskQueue::new())),
            progress: Arc::new(Mutex::new(BatchProgress::init(verbosity))),
        }
    }

    fn run(&mut self, command: &ImageCommand, args: &ImageArgs) -> Result<()> {
        self.progress.lock().unwrap().task_count(TASK_COUNT);

        self.decode_images(args)?;

        self.outpaths(args)?;

        self.save_images(args, command)?;

        self.progress.lock().unwrap().exit();

        Ok(())
    }

    fn decode_images(&mut self, args: &ImageArgs) -> Result<()> {
        let images = args.images.len() - 1;

        {
            let mut progress_lock = self.progress.lock().unwrap();

            progress_lock.sub_task_count(images);

            progress_lock.start_task(&format!("Deciding {} images", images));
        }

        let decoder_pool = ThreadPoolBuilder::new().num_threads(10).build().unwrap();

        decoder_pool.scope(|s| {
            for image in args.images.clone().iter_mut() {
                let tasks_queue = Arc::clone(&self.tasks_queue);

                let new_image = Arc::new(take(image));

                let progress = Arc::clone(&self.progress);

                let progress_lock = progress.lock().unwrap();

                progress_lock.start_sub_task(&format!(
                    "Decoding image: {:?}",
                    image.file_name().as_slice()
                ));

                drop(progress_lock);

                s.spawn(move |_| {
                    let new_image = new_image.to_path_buf();

                    let current_image = open_image(&new_image);

                    let mut queue_lock = tasks_queue.lock().unwrap();

                    let task_id = queue_lock.new_task(&new_image);

                    let mut progress_lock = progress.lock().unwrap();

                    match current_image {
                        Ok(mut good_image) => {
                            queue_lock.decoded_task(&mut good_image, task_id);
                            progress_lock.finish_sub_task(&format!(
                                "Image decoded: {:?}",
                                new_image.file_name().as_slice()
                            ));
                        }
                        Err(decode_error) => {
                            queue_lock.fail_task(task_id, decode_error.clone());
                            progress_lock.error_sub_task(&format!("{:?}", decode_error));
                        }
                    }
                });
            }
        });

        let progress_lock = self.progress.lock().unwrap();

        let queue_lock = self.tasks_queue.lock().unwrap();

        if queue_lock.has_failures() {
            progress_lock.finish_task(&format!(
                "{} images were decoded with {} errors",
                queue_lock.decoded_tasks().len(),
                queue_lock.count_failures()
            ));
            if args.abort_on_error {
                progress_lock.abort_task(&format!(
                    "Image processing exited with {} errros.",
                    queue_lock.count_failures()
                ));

                let mut errors = Vec::new();

                queue_lock.failed_tasks().iter().for_each(|task| {
                    if let TaskState::Failed(error) = &task.state {
                        errors.push(error.to_string());
                    }
                });

                return Err(TaskError::BatchError(errors).into());
            }
        } else {
            progress_lock.finish_task(&format!(
                "{} images were decoded successfully ^.^",
                queue_lock.decoded_tasks().len()
            ));
        }
        Ok(())
    }

    fn outpaths(&mut self, args: &ImageArgs) -> Result<()> {
        let progress_lock = self.progress.lock().unwrap();

        progress_lock.start_task("Creating output paths");
        let mut task_lock = self.tasks_queue.lock().unwrap();

        let destination_path = match &args.output {
            Some(path) => path.to_path_buf(),
            None => PathBuf::from("."),
        };

        let paths: Vec<PathBuf> = task_lock
            .decoded_tasks()
            .iter()
            .map(|task| task.image_path.to_path_buf())
            .collect();

        let mut output_paths = match create_paths(
            paths,
            destination_path,
            args.name_expr.as_deref(),
            args.format.as_deref(),
        ) {
            Ok(out_paths) => out_paths,
            Err(path_create_error) => {
                progress_lock.abort_task("Error creating out paths");
                return Err(TaskError::SingleError(path_create_error).into());
            }
        };

        match paths_exist(&output_paths) {
            Ok(paths) => {
                if !paths.is_empty() && !args.overwrite {
                    let prompt = || -> Result<()> {
                        match prompt_overwrite(paths) {
                            Ok(()) => Ok(()),
                            Err(error) => Err(TaskError::SingleError(error).into()),
                        }
                    };
                    progress_lock.suspend(|| -> Result<()> { prompt() })?;
                }
            }
            Err(e) => return Err(TaskError::SingleError(e).into()),
        }

        let ids = task_lock.decoded_task_ids().to_owned();
        for (index, path) in output_paths.iter_mut().enumerate() {
            task_lock.set_task_out_path(ids[index], path);
        }

        progress_lock.finish_task("Paths created successfully.");

        Ok(())
    }

    fn save_images(&mut self, args: &ImageArgs, command: &ImageCommand) -> Result<()> {
        let mut task_lock = self.tasks_queue.lock().unwrap();

        let mut progress_lock = self.progress.lock().unwrap();

        let decoded_tasks = task_lock.decoded_tasks().len();

        progress_lock.start_task(format!("Processing {} images", decoded_tasks).as_str());

        progress_lock.sub_task_count(decoded_tasks);

        for _ in 0..decoded_tasks {
            let task_id = task_lock.decoded_task_ids()[0];

            let mut current_task = {
                match task_lock.task_by_id_mut(task_id) {
                    Some(task) => std::mem::take(task),
                    _ => return Err(TaskError::NoSuchTask.into()),
                }
            };

            match command {
                ImageCommand::Convert => match &args.format {
                    Some(format) => progress_lock.start_sub_task(&format!(
                        "Coverting image: {} to format {}",
                        current_task.image_path.to_string_lossy(),
                        format
                    )),
                    None => progress_lock.start_sub_task(&format!(
                        "Converting image: {} as image {}",
                        current_task.image_path.to_string_lossy(),
                        current_task.out_path.to_string_lossy()
                    )),
                },
                ImageCommand::Resize(args) => {
                    progress_lock.start_sub_task(&format!(
                        "Resizing image: {}",
                        current_task.image_path.to_string_lossy()
                    ));
                    match args.run(&mut current_task.image) {
                        Ok(()) => {
                            task_lock.processed_task(&mut current_task.image, task_id);
                        }
                        Err(resize_error) => {
                            task_lock.fail_task(task_id, resize_error.to_string());
                            progress_lock.error_sub_task("Image resize exited with error.");
                        }
                    }
                }
                ImageCommand::Recolor(args) => {
                    progress_lock.start_sub_task(&format!(
                        "Recoloring image: {}",
                        current_task.image_path.to_string_lossy()
                    ));
                    match args.run(&mut current_task.image) {
                        Ok(()) => {
                            task_lock.processed_task(&mut current_task.image, task_id);
                        }
                        Err(recolor_error) => {
                            task_lock.fail_task(task_id, recolor_error.to_string());
                            progress_lock.error_sub_task("Image recolor exited with error.")
                        }
                    }
                }
                ImageCommand::Transparentize(args) => {
                    progress_lock.start_sub_task(&format!(
                        "Removing background: {}",
                        current_task.image_path.to_string_lossy()
                    ));
                    match args.run(&mut current_task.image) {
                        Ok(()) => {
                            task_lock.processed_task(&mut current_task.image, task_id);
                        }
                        Err(removal_error) => {
                            task_lock.fail_task(task_id, removal_error.to_string());
                            progress_lock.error_sub_task("Background removal exited with error.");
                        }
                    }
                }
            };

            if let Some(task) = task_lock.task_by_id(task_id) {
                if let TaskState::Failed(_) = task.state {
                    continue;
                }
            }

            progress_lock.start_sub_task(
                format!(
                    "Saving image: {:?}",
                    &current_task.out_path.file_name().as_slice()
                )
                .as_str(),
            );

            let image_result = save_image_format(
                &current_task.image,
                &current_task.out_path,
                args.format.as_deref(),
            );

            match image_result {
                Ok(()) => {
                    task_lock.completed_task(task_id);
                    progress_lock.finish_sub_task(&format!(
                        "Image saved successfully: {:?}",
                        &current_task.out_path.file_name().as_slice()
                    ));
                }
                Err(save_error) => {
                    task_lock.fail_task(task_id, save_error.to_string());
                    progress_lock.error_sub_task(&format!(
                        "Image processing failed with error: {:?}",
                        &current_task.out_path.file_name().as_slice(),
                    ));
                    progress_lock.send_trace(&format!(
                        "Image {:?} failed to process due to error: {}",
                        &current_task.out_path.file_name().as_slice(),
                        save_error
                    ));
                }
            }
        }
        Ok(())
    }
}

impl RunBatch for ImageArgs {
    fn run_batch(&self, command: &ImageCommand, verbosity: u32) -> Result<()> {
        BatchRunner::init(verbosity).run(command, self)
    }
}
