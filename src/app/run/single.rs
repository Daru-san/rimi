use super::RunSingle;
use crate::app::command::{ImageArgs, ImageCommand};
use crate::app::run::{command_msg, run_command};
use crate::backend::error::TaskError;
use crate::backend::paths::prompt_overwrite_single;
use crate::backend::progress::AppProgress;
use crate::backend::progress::SingleProgress;
use crate::image::manipulator::{open_image, save_image_format};

impl RunSingle for ImageArgs {
    fn run_single(&self, command: &ImageCommand, verbosity: u32) -> anyhow::Result<()> {
        let image_path = &self.images[0];

        let progress = SingleProgress::init(verbosity);

        const TASK_COUNT: usize = 4;

        progress.task_count(TASK_COUNT);

        progress.start_task(&format!(
            "Decoding image: {}",
            image_path.to_path_buf().to_string_lossy()
        ));

        let mut image = match open_image(image_path) {
            Ok(image) => {
                progress.finish_task("Image decoded successfully");
                image
            }
            Err(decode_error) => {
                progress.abort_task("Image decode failed");
                return Err(TaskError::SingleError(decode_error).into());
            }
        };

        let output_path = match &self.output {
            Some(path) => path,
            None => image_path,
        };

        match output_path.try_exists() {
            Ok(path_exists) => {
                if path_exists && !self.overwrite {
                    progress.suspend(|| -> Result<(), TaskError> {
                        match prompt_overwrite_single(output_path) {
                            Ok(()) => Ok(()),
                            Err(error) => Err(TaskError::SingleError(error)),
                        }
                    })?;
                }
            }
            Err(error) => return Err(error.into()),
        }

        progress.finish_task(&format!(
            "Set output path: {}",
            output_path.to_path_buf().to_string_lossy()
        ));

        progress.start_task(
            &command_msg(command, image_path.file_name().unwrap().to_str().unwrap()).unwrap(),
        );

        let image = match run_command(command, &mut image, self.format.as_deref()) {
            Ok(good_image) => good_image,
            Err(error) => return Err(error),
        };

        progress.start_task(&format!(
            "Saving image: {}",
            output_path.to_path_buf().to_string_lossy()
        ));

        match save_image_format(&image, output_path, self.format.as_deref()) {
            Ok(()) => progress.finish_task("Image saved successfully"),
            Err(save_error) => {
                progress.abort_task("Image failed to save");
                return Err(TaskError::SingleError(save_error).into());
            }
        }
        progress.exit();
        Ok(())
    }
}
