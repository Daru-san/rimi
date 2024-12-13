use std::path::PathBuf;

use crate::app::command::{Command, CommandArgs};
use crate::backend::error::TaskError;
use crate::backend::progress::{AppProgress, BatchProgress};
use crate::backend::queue::{TaskQueue, TaskState};

use super::RunBatch;
use anyhow::Result;

impl RunBatch for CommandArgs {
    fn run_batch(&self) -> Result<()> {
        use crate::utils::batch::*;
        use crate::utils::image::{open_image, save_image_format};

        let num_images = self.images.len() - 1;

        let mut tasks_queue = TaskQueue::new();
        let batch_progress = BatchProgress::init();

        batch_progress.start_sub_operation(format!("Deciding {} images", num_images).as_str());

        for (index, image) in self.images.iter().enumerate() {
            let task_id = tasks_queue.new_task(image);

            batch_progress.start_sub_operation(
                format!(
                    "[{}/{}] Decoding image: {:?}",
                    index,
                    num_images,
                    image.file_name().as_slice()
                )
                .as_str(),
            );

            let current_image = open_image(image.to_path_buf());

            match current_image {
                Ok(good_image) => {
                    tasks_queue.set_decoded(&good_image, task_id);
                }
                Err(error) => {
                    batch_progress.error_sub_operation(
                        format!(
                            "[{}/{}] Failed to decode image: {:?}",
                            index,
                            num_images,
                            image.file_name().as_slice()
                        )
                        .as_str(),
                    );
                    tasks_queue.set_failed(task_id, error);
                }
            }
            batch_progress.next_sub();
        }

        if tasks_queue.has_failures() {
            batch_progress.abort_message(
                format!(
                    "{} images were decoded with {} errors",
                    num_images,
                    tasks_queue.count_failures()
                )
                .as_str(),
            );
            if self.abort_on_error {
                let mut total_errors = Vec::new();
                for task in tasks_queue.failed_tasks().iter() {
                    if let TaskState::Failed(error) = &task.state {
                        total_errors.push(error.to_string());
                    }
                }

                return Err(TaskError::BatchError(total_errors).into());
            }
        } else {
            batch_progress.complete_operation_with_message(
                format!(
                    "{} images were decoded successfully ^.^",
                    tasks_queue.decoded_tasks().len()
                )
                .as_str(),
            );
        }

        batch_progress.start_operation("Creating output paths");

        let output_path = match &self.output {
            Some(path) => path.to_path_buf(),
            None => PathBuf::from("."),
        };

        let mut decoded_paths = Vec::new();
        for task in tasks_queue.decoded_tasks().iter() {
            decoded_paths.push(task.image_path.to_path_buf());
        }

        let out_paths = match create_paths(
            decoded_paths,
            output_path,
            self.extra_args.name_expr.as_deref(),
        ) {
            Ok(out_paths) => out_paths,
            Err(e) => return Err(TaskError::SingleError(e).into()),
        };

        for (index, path) in out_paths.iter().enumerate() {
            tasks_queue.set_out_path(tasks_queue.decoded_ids()[index], path);
        }

        batch_progress.complete_operation_with_message("Paths created successfully.");

        let count = tasks_queue.decoded_tasks().len();

        batch_progress.start_operation(format!("Processing {} images", count).as_str());

        for _ in 0..count {
            let task_id = tasks_queue.decoded_tasks()[0].id;

            let mut current_task = {
                match tasks_queue.task_by_id_mut(task_id) {
                    Some(task) => task.clone(),
                    _ => {
                        return Err(TaskError::SingleError("Task does not exist".to_string()).into())
                    }
                }
            };

            match &self.command {
                Command::Convert => match &self.extra_args.format {
                    Some(format) => batch_progress.complete_sub_operation(
                        format!(
                            "Coverting image: {} to format {}",
                            current_task.image_path.to_string_lossy(),
                            format
                        )
                        .as_str(),
                    ),
                    None => batch_progress.complete_sub_operation(
                        format!(
                            "Converting image: {} as image {}",
                            current_task.image_path.to_string_lossy(),
                            current_task.out_path.to_string_lossy()
                        )
                        .as_str(),
                    ),
                },
                Command::Resize(args) => {
                    batch_progress.start_sub_operation(
                        format!(
                            "Resizing image: {}",
                            current_task.image_path.to_string_lossy()
                        )
                        .as_str(),
                    );
                    match args.run(&mut current_task.image) {
                        Ok(()) => {
                            tasks_queue.set_processed(&current_task.image, task_id);
                            batch_progress.complete_sub_operation("Image resized successfully");
                        }
                        Err(e) => {
                            tasks_queue.set_failed(task_id, e.to_string());
                            batch_progress.error_sub_operation("Image resize exited with error.");
                        }
                    }
                }
                Command::Recolor(args) => {
                    batch_progress.start_sub_operation(
                        format!(
                            "Recoloring image: {}",
                            current_task.image_path.to_string_lossy()
                        )
                        .as_str(),
                    );
                    match args.run(&mut current_task.image) {
                        Ok(()) => {
                            tasks_queue.set_processed(&current_task.image, task_id);
                            batch_progress.complete_sub_operation("Image recolored successfully.");
                        }
                        Err(e) => {
                            tasks_queue.set_failed(task_id, e.to_string());
                            batch_progress.error_sub_operation("Image recolor exited with error.")
                        }
                    }
                }
                Command::Transparentize(args) => {
                    batch_progress.start_sub_operation(
                        format!(
                            "Removing background: {}",
                            current_task.image_path.to_string_lossy()
                        )
                        .as_str(),
                    );
                    match args.run(&mut current_task.image) {
                        Ok(()) => {
                            tasks_queue.set_processed(&current_task.image, task_id);
                            batch_progress.complete_sub_operation("Image background removed.");
                        }
                        Err(e) => {
                            tasks_queue.set_failed(task_id, e.to_string());
                            batch_progress
                                .error_sub_operation("Background removal exited with error.");
                        }
                    }
                }
                command => {
                    return Err(TaskError::SingleError(format!(
                        "{:?} command cannot be run here.",
                        command
                    ))
                    .into());
                }
            };

            if let Some(task) = tasks_queue.task_by_id(task_id) {
                if let TaskState::Failed(_) = task.state {
                    continue;
                }
            }

            batch_progress.start_sub_operation(
                format!(
                    "Saving image: {:?}",
                    &current_task.out_path.file_name().as_slice()
                )
                .as_str(),
            );

            let image_result = save_image_format(
                &current_task.image,
                &current_task.out_path,
                self.extra_args.format.as_deref(),
                self.extra_args.overwrite,
            );

            match image_result {
                Ok(()) => {
                    tasks_queue.set_completed(task_id);
                    batch_progress.start_sub_operation(
                        format!(
                            "Image processing complete: {:?}",
                            &current_task.out_path.file_name().as_slice()
                        )
                        .as_str(),
                    );
                }
                Err(e) => {
                    tasks_queue.set_failed(task_id, e.to_string());
                    batch_progress.error_sub_operation(
                        format!(
                            "Image processing failed with error: {:?}",
                            &current_task.out_path.file_name().as_slice()
                        )
                        .as_str(),
                    );
                }
            }
        }
        if !tasks_queue.completed_tasks().is_empty() {
            batch_progress.complete();
        }
        Ok(())
    }
}
