use std::path::PathBuf;

use crate::app::command::{ImageArgs, ImageCommand};
use crate::backend::error::TaskError;
use crate::backend::paths::{create_paths, paths_exist, prompt_overwrite};
use crate::backend::progress::{AppProgress, BatchProgress};
use crate::backend::queue::{TaskQueue, TaskState};
use crate::image::manipulator::{open_image, save_image_format};

use super::RunBatch;
use anyhow::Result;

const TASK_COUNT: usize = 3;

struct BatchRunner {
    tasks_queue: TaskQueue,
    progress: BatchProgress,
}

impl BatchRunner {
    fn init(verbosity: u32) -> Self {
        Self {
            tasks_queue: TaskQueue::new(),
            progress: BatchProgress::init(verbosity),
        }
    }

    fn run(&mut self, command: &ImageCommand, args: &ImageArgs) -> Result<()> {
        self.progress.task_count(TASK_COUNT);

        self.decode_images(args)?;

        self.outpaths(args)?;

        self.save_images(args, command)?;

        self.progress.exit();

        Ok(())
    }

    fn decode_images(&mut self, args: &ImageArgs) -> Result<()> {
        let images = args.images.len() - 1;
        self.progress.sub_task_count(images);
        self.progress
            .start_task(&format!("Deciding {} images", images));

        let images = &args.images;
        for image in images.iter() {
            let task_id = self.tasks_queue.new_task(image);

            self.progress.start_sub_task(&format!(
                "Decoding image: {:?}",
                image.file_name().as_slice()
            ));

            let current_image = open_image(image);

            match current_image {
                Ok(mut good_image) => {
                    self.tasks_queue.decoded_task(&mut good_image, task_id);
                    self.progress.finish_sub_task(&format!(
                        "Image decoded: {}",
                        image.as_path().to_string_lossy()
                    ));
                }
                Err(decode_error) => {
                    self.progress.error_sub_task(decode_error.as_str());
                    self.tasks_queue.fail_task(task_id, decode_error);
                }
            }
        }

        if self.tasks_queue.has_failures() {
            self.progress.finish_task(&format!(
                "{} images were decoded with {} errors",
                self.tasks_queue.decoded_tasks().len(),
                self.tasks_queue.count_failures()
            ));
            if args.abort_on_error {
                self.progress.abort_task(&format!(
                    "Image processing exited with {} errros.",
                    self.tasks_queue.count_failures()
                ));

                let mut errors = Vec::new();

                self.tasks_queue.failed_tasks().iter().for_each(|task| {
                    if let TaskState::Failed(error) = &task.state {
                        errors.push(error.to_string());
                    }
                });

                return Err(TaskError::BatchError(errors).into());
            }
        } else {
            self.progress.finish_task(&format!(
                "{} images were decoded successfully ^.^",
                self.tasks_queue.decoded_tasks().len()
            ));
        }
        Ok(())
    }

    fn outpaths(&mut self, args: &ImageArgs) -> Result<()> {
        self.progress.start_task("Creating output paths");

        let destination_path = match &args.output {
            Some(path) => path.to_path_buf(),
            None => PathBuf::from("."),
        };

        let paths: Vec<PathBuf> = self
            .tasks_queue
            .decoded_tasks()
            .iter()
            .map(|task| task.image_path.to_path_buf())
            .collect();

        let output_paths = match create_paths(
            paths,
            destination_path,
            args.name_expr.as_deref(),
            args.format.as_deref(),
        ) {
            Ok(out_paths) => out_paths,
            Err(path_create_error) => {
                self.progress.abort_task("Error creating out paths");
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
                    self.progress.suspend(|| -> Result<()> { prompt() })?;
                }
            }
            Err(e) => return Err(TaskError::SingleError(e).into()),
        }

        for (index, path) in output_paths.iter().enumerate() {
            self.tasks_queue
                .set_task_out_path(self.tasks_queue.decoded_task_ids()[index], path);
        }

        self.progress.finish_task("Paths created successfully.");

        Ok(())
    }

    fn save_images(&mut self, args: &ImageArgs, command: &ImageCommand) -> Result<()> {
        let decoded_tasks = self.tasks_queue.decoded_tasks().len();

        self.progress
            .start_task(format!("Processing {} images", decoded_tasks).as_str());

        self.progress.sub_task_count(decoded_tasks);

        for _ in 0..decoded_tasks {
            let task_id = self.tasks_queue.decoded_task_ids()[0];

            let mut current_task = {
                match self.tasks_queue.task_by_id_mut(task_id) {
                    Some(task) => task.clone(),
                    _ => return Err(TaskError::NoSuchTask.into()),
                }
            };

            match command {
                ImageCommand::Convert => match &args.format {
                    Some(format) => self.progress.start_sub_task(&format!(
                        "Coverting image: {} to format {}",
                        current_task.image_path.to_string_lossy(),
                        format
                    )),
                    None => self.progress.start_sub_task(&format!(
                        "Converting image: {} as image {}",
                        current_task.image_path.to_string_lossy(),
                        current_task.out_path.to_string_lossy()
                    )),
                },
                ImageCommand::Resize(args) => {
                    self.progress.start_sub_task(&format!(
                        "Resizing image: {}",
                        current_task.image_path.to_string_lossy()
                    ));
                    match args.run(&mut current_task.image) {
                        Ok(()) => {
                            self.tasks_queue
                                .processed_task(&mut current_task.image, task_id);
                        }
                        Err(resize_error) => {
                            self.tasks_queue
                                .fail_task(task_id, resize_error.to_string());
                            self.progress
                                .error_sub_task("Image resize exited with error.");
                        }
                    }
                }
                ImageCommand::Recolor(args) => {
                    self.progress.start_sub_task(&format!(
                        "Recoloring image: {}",
                        current_task.image_path.to_string_lossy()
                    ));
                    match args.run(&mut current_task.image) {
                        Ok(()) => {
                            self.tasks_queue
                                .processed_task(&mut current_task.image, task_id);
                        }
                        Err(recolor_error) => {
                            self.tasks_queue
                                .fail_task(task_id, recolor_error.to_string());
                            self.progress
                                .error_sub_task("Image recolor exited with error.")
                        }
                    }
                }
                ImageCommand::Transparentize(args) => {
                    self.progress.start_sub_task(&format!(
                        "Removing background: {}",
                        current_task.image_path.to_string_lossy()
                    ));
                    match args.run(&mut current_task.image) {
                        Ok(()) => {
                            self.tasks_queue
                                .processed_task(&mut current_task.image, task_id);
                        }
                        Err(removal_error) => {
                            self.tasks_queue
                                .fail_task(task_id, removal_error.to_string());
                            self.progress
                                .error_sub_task("Background removal exited with error.");
                        }
                    }
                }
            };

            if let Some(task) = self.tasks_queue.task_by_id(task_id) {
                if let TaskState::Failed(_) = task.state {
                    continue;
                }
            }

            self.progress.start_sub_task(
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
                    self.tasks_queue.completed_task(task_id);
                    self.progress.finish_sub_task(&format!(
                        "Image saved successfully: {:?}",
                        &current_task.out_path.file_name().as_slice()
                    ));
                }
                Err(save_error) => {
                    self.tasks_queue.fail_task(task_id, save_error.to_string());
                    self.progress.error_sub_task(&format!(
                        "Image processing failed with error: {:?}",
                        &current_task.out_path.file_name().as_slice(),
                    ));
                    self.progress.send_trace(&format!(
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
