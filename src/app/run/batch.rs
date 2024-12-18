use std::mem::take;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use crate::app::command::{ImageArgs, ImageCommand};
use crate::backend::error::TaskError;
use crate::backend::paths::{create_paths, paths_exist, prompt_overwrite};
use crate::backend::progress::{AppProgress, BatchProgress};
use crate::backend::queue::{TaskQueue, TaskState};
use crate::image::manipulator::{convert_image, open_image, save_image_format};

use super::RunBatch;
use anyhow::{Error, Result};
use image::DynamicImage;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use rayon::{ThreadPool, ThreadPoolBuilder};

const TASK_COUNT: usize = 3;

struct BatchRunner {
    tasks_queue: Arc<Mutex<TaskQueue>>,
    progress: Arc<Mutex<BatchProgress>>,
    tasks_pool: ThreadPool,
}

impl BatchRunner {
    fn init(verbosity: u32, task_count: usize) -> Self {
        Self {
            tasks_queue: Arc::new(Mutex::new(TaskQueue::new())),
            progress: Arc::new(Mutex::new(BatchProgress::init(verbosity))),
            tasks_pool: ThreadPoolBuilder::new()
                .num_threads(task_count)
                .thread_name(|i| format!("Pool thread {i}"))
                .build()
                .unwrap(),
        }
    }

    fn run(&mut self, command: &ImageCommand, args: &ImageArgs) -> Result<()> {
        self.progress.lock().unwrap().task_count(TASK_COUNT);

        self.decode_images(args)?;

        self.outpaths(args)?;

        self.process_images(command, args.format.as_deref())?;

        self.save_images(args)?;

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

        let tasks_queue = &self.tasks_queue;

        let progress = &self.progress;

        args.images.par_iter().for_each(|image| {
            let progress_lock = progress.lock().unwrap();

            progress_lock.start_sub_task(&format!(
                "Decoding image: {:?}",
                image.file_name().as_slice()
            ));

            drop(progress_lock);

            let current_image = open_image(image);

            let mut queue_lock = tasks_queue.lock().unwrap();

            let task_id = queue_lock.new_task(image);

            let mut progress_lock = progress.lock().unwrap();

            match current_image {
                Ok(mut good_image) => {
                    queue_lock.decoded_task(&mut good_image, task_id);
                    progress_lock.finish_sub_task(&format!(
                        "Image decoded: {:?}",
                        image.file_name().as_slice()
                    ));
                }
                Err(decode_error) => {
                    queue_lock.fail_task(task_id, decode_error.clone());
                    progress_lock.error_sub_task(&format!("{:?}", decode_error));
                }
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

    fn process_images(&mut self, command: &ImageCommand, format: Option<&str>) -> Result<()> {
        self.tasks_pool.scope(|s| -> Result<()> {
            let has_images = Arc::new(AtomicBool::new(true));

            while has_images.load(Ordering::Relaxed) {
                let tasks_queue = Arc::clone(&self.tasks_queue);

                let progress = Arc::clone(&self.progress);

                let has_images: Arc<AtomicBool> = Arc::clone(&has_images);

                s.spawn(move |_| {
                    let (task_id, mut processed_image, out_path) = {
                        let mut task_lock = tasks_queue.lock().unwrap();

                        let task_id = if !task_lock.decoded_tasks().is_empty() {
                            task_lock.decoded_tasks()[0].id
                        } else {
                            has_images.store(false, Ordering::Relaxed);
                            return;
                        };

                        let task_data = match task_lock.task_by_id_mut(task_id) {
                            Some(task) => match task.state {
                                TaskState::Decoded => {
                                    (task.id, take(&mut task.image), take(&mut task.image_path))
                                }
                                TaskState::Failed(_) => return,
                                _ => return,
                            },
                            _ => {
                                task_lock.fail_task(task_id, TaskError::NoSuchTask.to_string());
                                return;
                            }
                        };

                        task_lock.working_task(task_data.0);

                        task_data
                    };

                    {
                        let progress_lock = progress.lock().unwrap();
                        progress_lock.start_task(
                            &command_msg(command, out_path.to_string_lossy().to_string().as_ref())
                                .unwrap(),
                        );
                    }

                    let result = run_command(command, &mut processed_image, format);

                    let mut task_lock = tasks_queue.lock().unwrap();

                    let mut progress_lock = progress.lock().unwrap();

                    match result {
                        Ok(mut image) => task_lock.processed_task(&mut image, task_id),
                        Err(error) => {
                            progress_lock.error_sub_task(&format!("Error: {}", error));
                            task_lock.fail_task(task_id, error.to_string());
                        }
                    };
                });
            }
            Ok(())
        })?;
        Ok(())
    }
    fn save_images(&mut self, args: &ImageArgs) -> Result<()> {
        self.tasks_pool.in_place_scope(|s| {
            let has_images = Arc::new(AtomicBool::new(true));

            while has_images.load(Ordering::Relaxed) {
                let tasks_queue = Arc::clone(&self.tasks_queue);
                let progress = Arc::clone(&self.progress);

                let has_images: Arc<AtomicBool> = Arc::clone(&has_images);

                s.spawn(move |_| {
                    let (task_id, processed_image, out_path) = {
                        let mut task_lock = tasks_queue.lock().unwrap();

                        let task_id = if !task_lock.processed_tasks().is_empty() {
                            task_lock.processed_tasks()[0].id
                        } else {
                            has_images.store(false, Ordering::Relaxed);
                            return;
                        };

                        let task_data = match task_lock.task_by_id_mut(task_id) {
                            Some(task) => match task.state {
                                TaskState::Processed => {
                                    (task.id, take(&mut task.image), take(&mut task.out_path))
                                }
                                TaskState::Failed(_) => return,
                                _ => return,
                            },
                            _ => {
                                task_lock.fail_task(task_id, TaskError::NoSuchTask.to_string());
                                return;
                            }
                        };

                        task_lock.working_task(task_data.0);

                        task_data
                    };

                    {
                        let progress_lock = progress.lock().unwrap();
                        progress_lock.start_task(&format!(
                            "Saving image: {:?}",
                            out_path.file_name().as_slice()
                        ));
                    }

                    let image_result =
                        save_image_format(&processed_image, &out_path, args.format.as_deref());

                    let mut progress_lock = progress.lock().unwrap();
                    let mut task_lock = tasks_queue.lock().unwrap();

                    match image_result {
                        Ok(()) => {
                            task_lock.completed_task(task_id);
                            progress_lock.finish_sub_task(&format!(
                                "Image saved successfully: {:?}",
                                out_path
                            ));
                        }
                        Err(save_error) => {
                            task_lock.fail_task(task_id, save_error.to_string());
                            progress_lock.error_sub_task(&format!(
                                "Image processing failed with error: {:?}",
                                out_path
                            ));
                            progress_lock.send_trace(&format!(
                                "Image {:?} failed to process due to error: {}",
                                out_path, save_error
                            ));
                        }
                    }
                });
            }
        });
        Ok(())
    }
}

fn run_command(
    command: &ImageCommand,
    image: &mut DynamicImage,
    format: Option<&str>,
) -> Result<DynamicImage> {
    match command {
        ImageCommand::Convert => match convert_image(image, format) {
            Ok(mut image) => Ok(take(&mut image)),
            Err(convert_error) => Err(Error::msg(convert_error)),
        },
        ImageCommand::Resize(args) => match args.run(image) {
            Ok(()) => Ok(take(image)),
            Err(resize_error) => Err(resize_error),
        },
        ImageCommand::Recolor(args) => match args.run(image) {
            Ok(()) => Ok(take(image)),
            Err(recolor_error) => Err(recolor_error),
        },
        ImageCommand::Transparentize(args) => match args.run(image) {
            Ok(()) => Ok(take(image)),
            Err(removal_error) => Err(removal_error),
        },
    }
}

fn command_msg(command: &ImageCommand, image_name: &str) -> Result<String> {
    let message = match command {
        ImageCommand::Convert => "Converting ",
        ImageCommand::Resize(_) => "Resizing ",
        ImageCommand::Recolor(_) => "Recoloring ",
        ImageCommand::Transparentize(_) => "Removing background ",
    };
    Ok(format!("{message} {image_name}"))
}

impl RunBatch for ImageArgs {
    fn run_batch(&self, command: &ImageCommand, verbosity: u32, task_count: usize) -> Result<()> {
        BatchRunner::init(verbosity, task_count).run(command, self)
    }
}
