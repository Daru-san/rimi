use std::mem::take;
use std::path::PathBuf;
use std::sync::Mutex;

use crate::app::command::{ImageArgs, ImageCommand};
use crate::backend::error::TaskError;
use crate::backend::paths::{create_paths, paths_exist, prompt_overwrite};
use crate::backend::progress::{AppProgressBar, BatchProgressBar};
use crate::backend::queue::{TaskQueue, TaskState};
use crate::image::manipulator::{open_image, save_image_format};

use super::{command_msg, run_command, RunBatch};
use anyhow::Result;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

const TASK_COUNT: usize = 4;

struct BatchRunner {
    tasks_queue: Mutex<TaskQueue>,
    progress_bar: Mutex<BatchProgressBar>,
}

impl BatchRunner {
    fn init(verbosity: u32) -> Self {
        Self {
            tasks_queue: Mutex::new(TaskQueue::new()),
            progress_bar: Mutex::new(BatchProgressBar::init(verbosity, TASK_COUNT)),
        }
    }

    fn run(&mut self, command: &ImageCommand, args: &ImageArgs) -> Result<()> {
        self.decode_images(args)?;

        self.outpaths(args)?;

        self.process_images(command, args.format.as_deref())?;

        self.save_images(args)?;

        self.progress_bar.lock().unwrap().exit();

        Ok(())
    }

    fn decode_images(&mut self, args: &ImageArgs) -> Result<()> {
        let tasks_queue = &self.tasks_queue;
        let progress_bar = &self.progress_bar;

        let images = args.images.len() - 1;
        {
            let progress_bar = progress_bar.lock().unwrap();

            progress_bar.spawn_new(images, &format!("Deciding {} images", images));
        }

        args.images.par_iter().for_each(|image| {
            if let Ok(progress_bar) = progress_bar.try_lock() {
                progress_bar.start_task(&format!(
                    "Decoding image: {:?}",
                    image.file_name().as_slice()
                ));
            }

            let current_image = open_image(image);

            let mut tasks_queue = tasks_queue.lock().unwrap();

            let task_id = tasks_queue.new_task(image);

            match current_image {
                Ok(good_image) => tasks_queue.decoded_task(&mut Some(good_image), task_id),
                Err(decode_error) => {
                    tasks_queue.fail_task(task_id, &decode_error);
                    if let Ok(progress) = progress_bar.try_lock() {
                        progress.send_trace(&format!("Error: {}", decode_error));
                    }
                }
            }
        });

        let tasks_queue = tasks_queue.lock().unwrap();
        let progress_bar = progress_bar.lock().unwrap();

        if tasks_queue.has_failures() {
            progress_bar.message(&format!(
                "{} images were decoded with {} errors",
                tasks_queue.decoded_tasks().len(),
                tasks_queue.count_failures()
            ));
            if args.abort_on_error {
                progress_bar.abort(&format!(
                    "Image processing exited with {} errros.",
                    tasks_queue.count_failures()
                ));

                let mut errors = Vec::new();

                tasks_queue.failed_tasks().iter().for_each(|task| {
                    if let TaskState::Failed(error) = &task.state {
                        errors.push(error.to_string());
                    }
                });

                return Err(TaskError::BatchError(errors).into());
            }
        } else {
            progress_bar.message(&format!(
                "{} images were decoded successfully ^.^",
                tasks_queue.decoded_tasks().len()
            ));
        }
        Ok(())
    }

    fn outpaths(&mut self, args: &ImageArgs) -> Result<()> {
        let progress_bar = self.progress_bar.lock().unwrap();
        let mut tasks_queue = self.tasks_queue.lock().unwrap();

        progress_bar.spawn_new(1, "Creating output paths");

        let destination_path = match &args.output {
            Some(path) => path.to_path_buf(),
            None => PathBuf::from("."),
        };

        let paths: Vec<PathBuf> = tasks_queue
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
                progress_bar.abort("Error creating out paths");
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
                    progress_bar.suspend(|| -> Result<()> { prompt() })?;
                }
            }
            Err(e) => return Err(TaskError::SingleError(e).into()),
        }

        let ids = tasks_queue.decoded_task_ids().to_owned();
        for (index, path) in output_paths.iter_mut().enumerate() {
            tasks_queue.set_task_out_path(ids[index], path);
        }

        progress_bar.message("Paths created successfully.");

        Ok(())
    }

    fn process_images(&mut self, command: &ImageCommand, format: Option<&str>) -> Result<()> {
        let tasks_queue = &self.tasks_queue;
        let progress = &self.progress_bar;

        let task_ids: Vec<u32> = {
            let tasks_queue = tasks_queue.lock().unwrap();
            tasks_queue
                .decoded_tasks()
                .par_iter()
                .map(|task| task.id)
                .collect()
        };

        {
            let progress = progress.lock().unwrap();
            progress.spawn_new(
                task_ids.len(),
                &format!("Processing {} images", task_ids.len()),
            );
        }

        task_ids.par_iter().for_each(|task_id| {
            let (task_id, mut processed_image, file_path) = {
                let mut tasks_queue = match tasks_queue.lock() {
                    Ok(lock) => lock,
                    Err(error) => {
                        panic!("A serious error occured! {}", error);
                    }
                };

                let (id, image, path) = match tasks_queue.task_by_id_mut(*task_id) {
                    Some(task) => match task.state {
                        TaskState::Decoded => {
                            (task.id, task.image.take(), take(&mut task.image_path))
                        }
                        TaskState::Failed(_) => return,
                        _ => return,
                    },
                    _ => {
                        tasks_queue.fail_task(*task_id, &TaskError::NoSuchTask.to_string());
                        return;
                    }
                };

                tasks_queue.working_task(id);

                if let Some(image) = image {
                    (id, image, path)
                } else {
                    tasks_queue.fail_task(id, &TaskError::NoSuchTask.to_string());
                    return;
                }
            };

            if let Ok(progress_bar) = progress.try_lock() {
                progress_bar.start_task(
                    &command_msg(command, file_path.to_string_lossy().to_string().as_ref())
                        .unwrap(),
                );
            }

            let result = run_command(command, &mut processed_image, format);

            let mut task_guard = tasks_queue.lock().unwrap();

            match result {
                Ok(image) => task_guard.processed_task(&mut Some(image), task_id),
                Err(error) => {
                    task_guard.fail_task(task_id, &error.to_string());
                    if let Ok(progress) = progress.try_lock() {
                        progress.send_trace(&format!("Error: {}", error));
                    }
                }
            };
        });
        Ok(())
    }
    fn save_images(&mut self, args: &ImageArgs) -> Result<()> {
        let tasks_queue = &self.tasks_queue;
        let progress = &self.progress_bar;

        let task_ids: Vec<u32> = {
            let tasks_queue = tasks_queue.lock().unwrap();
            tasks_queue
                .processed_tasks()
                .par_iter()
                .map(|task| task.id)
                .collect()
        };

        {
            let progress = progress.lock().unwrap();
            progress.spawn_new(task_ids.len(), &format!("Saving {} images", task_ids.len()));
        }

        task_ids.par_iter().for_each(|task_id| {
            let (task_id, processed_image, out_path) = {
                let mut tasks_queue = match tasks_queue.lock() {
                    Ok(lock) => lock,
                    Err(error) => {
                        panic!("A serious error occured! {}", error);
                    }
                };

                let (id, image, path) = match tasks_queue.task_by_id_mut(*task_id) {
                    Some(task) => match task.state {
                        TaskState::Processed => {
                            (task.id, task.image.take(), take(&mut task.out_path))
                        }
                        TaskState::Failed(_) => return,
                        _ => return,
                    },
                    _ => {
                        tasks_queue.fail_task(*task_id, &TaskError::NoSuchTask.to_string());
                        return;
                    }
                };

                tasks_queue.working_task(id);

                if let Some(image) = image {
                    (id, image, path)
                } else {
                    tasks_queue.fail_task(id, &TaskError::NoSuchTask.to_string());
                    return;
                }
            };

            if let Ok(progress_bar) = progress.try_lock() {
                progress_bar.start_task(&format!(
                    "Saving image: {:?}",
                    out_path.file_name().as_slice()
                ));
            }

            let image_result =
                save_image_format(&processed_image, &out_path, args.format.as_deref());

            let mut task_guard = tasks_queue.lock().unwrap();

            match image_result {
                Ok(()) => task_guard.completed_task(task_id),
                Err(save_error) => {
                    task_guard.fail_task(task_id, &save_error);
                    if let Ok(progress) = progress.try_lock() {
                        progress.send_trace(&format!("Error: {}", save_error));
                    }
                }
            }
        });
        Ok(())
    }
}

impl RunBatch for ImageArgs {
    fn run_batch(&self, command: &ImageCommand, verbosity: u32) -> Result<()> {
        BatchRunner::init(verbosity).run(command, self)
    }
}
