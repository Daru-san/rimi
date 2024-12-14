use std::path::PathBuf;

use crate::app::command::{ImageArgs, ImageCommand};
use crate::backend::error::TaskError;
use crate::backend::paths::create_paths;
use crate::backend::progress::{AppProgress, BatchProgress};
use crate::backend::queue::{TaskQueue, TaskState};
use crate::image::manipulator::{open_image, save_image_format};

use super::RunBatch;
use anyhow::Result;

impl RunBatch for ImageArgs {
    fn run_batch(&self, command: &ImageCommand, verbosity: u32) -> Result<()> {
        let num_images = self.images.len() - 1;

        const TASK_COUNT: usize = 3;

        let mut tasks_queue = TaskQueue::new();

        let mut batch_progress = BatchProgress::init(verbosity);

        batch_progress.task_count(TASK_COUNT);

        batch_progress.start_task(format!("Deciding {} images", num_images).as_str());

        batch_progress.sub_task_count(num_images);

        for image in self.images.iter() {
            let task_id = tasks_queue.new_task(image);

            batch_progress.start_sub_task(
                format!("Decoding image: {:?}", image.file_name().as_slice()).as_str(),
            );

            let current_image = open_image(image);

            match current_image {
                Ok(good_image) => {
                    tasks_queue.decoded_task(&good_image, task_id);
                }
                Err(decode_error) => {
                    batch_progress.error_sub_task(decode_error.as_str());
                    tasks_queue.fail_task(task_id, decode_error);
                }
            }
        }

        if tasks_queue.has_failures() {
            batch_progress.finish_task(
                format!(
                    "{} images were decoded with {} errors",
                    num_images,
                    tasks_queue.count_failures()
                )
                .as_str(),
            );
            if self.abort_on_error {
                batch_progress.abort_task(
                    format!(
                        "Image processing exited with {} errros.",
                        tasks_queue.count_failures()
                    )
                    .as_str(),
                );
                let mut total_errors = Vec::new();
                for task in tasks_queue.failed_tasks().iter() {
                    if let TaskState::Failed(error) = &task.state {
                        total_errors.push(error.to_string());
                    }
                }

                return Err(TaskError::BatchError(total_errors).into());
            }
        } else {
            batch_progress.finish_task(
                format!(
                    "{} images were decoded successfully ^.^",
                    tasks_queue.decoded_tasks().len()
                )
                .as_str(),
            );
        }

        batch_progress.start_task("Creating output paths");

        let output_path = match &self.output {
            Some(path) => path.to_path_buf(),
            None => PathBuf::from("."),
        };

        let mut decoded_paths = Vec::new();
        for task in tasks_queue.decoded_tasks().iter() {
            decoded_paths.push(task.image_path.to_path_buf());
        }

        let out_paths = match create_paths(decoded_paths, output_path, self.name_expr.as_deref()) {
            Ok(out_paths) => out_paths,
            Err(path_create_error) => {
                batch_progress.abort_task("Error creating out paths");
                return Err(TaskError::SingleError(path_create_error).into());
            }
        };

        for (index, path) in out_paths.iter().enumerate() {
            tasks_queue.set_task_out_path(tasks_queue.decoded_task_ids()[index], path);
        }

        batch_progress.finish_task("Paths created successfully.");

        let decoded_tasks = tasks_queue.decoded_tasks().len();

        batch_progress.start_task(format!("Processing {} images", decoded_tasks).as_str());

        batch_progress.sub_task_count(decoded_tasks);

        for _ in 0..decoded_tasks {
            let task_id = tasks_queue.decoded_task_ids()[0];

            let mut current_task = {
                match tasks_queue.task_by_id_mut(task_id) {
                    Some(task) => task.clone(),
                    _ => return Err(TaskError::NoSuchTask.into()),
                }
            };

            match command {
                ImageCommand::Convert => match &self.format {
                    Some(format) => batch_progress.start_sub_task(
                        format!(
                            "Coverting image: {} to format {}",
                            current_task.image_path.to_string_lossy(),
                            format
                        )
                        .as_str(),
                    ),
                    None => batch_progress.start_sub_task(
                        format!(
                            "Converting image: {} as image {}",
                            current_task.image_path.to_string_lossy(),
                            current_task.out_path.to_string_lossy()
                        )
                        .as_str(),
                    ),
                },
                ImageCommand::Resize(args) => {
                    batch_progress.start_sub_task(
                        format!(
                            "Resizing image: {}",
                            current_task.image_path.to_string_lossy()
                        )
                        .as_str(),
                    );
                    match args.run(&mut current_task.image) {
                        Ok(()) => {
                            tasks_queue.processed_task(&current_task.image, task_id);
                        }
                        Err(resize_error) => {
                            tasks_queue.fail_task(task_id, resize_error.to_string());
                            batch_progress.error_sub_task("Image resize exited with error.");
                        }
                    }
                }
                ImageCommand::Recolor(args) => {
                    batch_progress.start_sub_task(
                        format!(
                            "Recoloring image: {}",
                            current_task.image_path.to_string_lossy()
                        )
                        .as_str(),
                    );
                    match args.run(&mut current_task.image) {
                        Ok(()) => {
                            tasks_queue.processed_task(&current_task.image, task_id);
                        }
                        Err(recolor_error) => {
                            tasks_queue.fail_task(task_id, recolor_error.to_string());
                            batch_progress.error_sub_task("Image recolor exited with error.")
                        }
                    }
                }
                ImageCommand::Transparentize(args) => {
                    batch_progress.start_sub_task(
                        format!(
                            "Removing background: {}",
                            current_task.image_path.to_string_lossy()
                        )
                        .as_str(),
                    );
                    match args.run(&mut current_task.image) {
                        Ok(()) => {
                            tasks_queue.processed_task(&current_task.image, task_id);
                        }
                        Err(removal_error) => {
                            tasks_queue.fail_task(task_id, removal_error.to_string());
                            batch_progress.error_sub_task("Background removal exited with error.");
                        }
                    }
                }
            };

            if let Some(task) = tasks_queue.task_by_id(task_id) {
                if let TaskState::Failed(_) = task.state {
                    continue;
                }
            }

            batch_progress.start_sub_task(
                format!(
                    "Saving image: {:?}",
                    &current_task.out_path.file_name().as_slice()
                )
                .as_str(),
            );

            let image_result = save_image_format(
                &current_task.image,
                &current_task.out_path,
                self.format.as_deref(),
                self.overwrite,
            );

            match image_result {
                Ok(()) => {
                    tasks_queue.completed_task(task_id);
                    batch_progress.finish_sub_task(
                        format!(
                            "Image saved successfully: {:?}",
                            &current_task.out_path.file_name().as_slice()
                        )
                        .as_str(),
                    );
                }
                Err(save_error) => {
                    tasks_queue.fail_task(task_id, save_error.to_string());
                    batch_progress.error_sub_task(
                        format!(
                            "Image processing failed with error: {:?}",
                            &current_task.out_path.file_name().as_slice(),
                        )
                        .as_str(),
                    );
                }
            }
        }
        batch_progress.exit();
        Ok(())
    }
}
