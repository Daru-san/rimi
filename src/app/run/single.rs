use super::RunSingle;
use crate::app::command::{ImageArgs, ImageCommand};
use crate::app::run::{command_msg, run_command};
use crate::backend::error::TaskError;
use crate::backend::paths::prompt_overwrite_single;
use crate::backend::progress::AppProgressBar;
use crate::backend::progress::SingleProgressBar;
use crate::image::manipulator::{open_image, save_image_format};

const TASK_COUNT: usize = 4;

impl RunSingle for ImageArgs {
    fn run_single(&self, command: &ImageCommand, verbosity: u32) -> anyhow::Result<()> {
        let image_path = &self.images[0];

        let progress_bar = SingleProgressBar::init(verbosity, TASK_COUNT);

        progress_bar.start_task(&format!(
            "Decoding image: {}",
            image_path.to_path_buf().to_string_lossy()
        ));

        let mut image = match open_image(image_path) {
            Ok(image) => {
                progress_bar.message("Image decoded successfully");
                image
            }
            Err(decode_error) => {
                progress_bar.abort("Image decode failed");
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
                    progress_bar.suspend(|| -> Result<(), TaskError> {
                        match prompt_overwrite_single(output_path) {
                            Ok(()) => Ok(()),
                            Err(error) => Err(TaskError::SingleError(error)),
                        }
                    })?;
                }
            }
            Err(error) => return Err(error.into()),
        }

        progress_bar.message(&format!(
            "Set output path: {}",
            output_path.to_path_buf().to_string_lossy()
        ));

        progress_bar.start_task(
            &command_msg(command, image_path.file_name().unwrap().to_str().unwrap()).unwrap(),
        );

        let image = match run_command(command, &mut image, self.format.as_deref()) {
            Ok(good_image) => good_image,
            Err(error) => return Err(error),
        };

        progress_bar.start_task(&format!(
            "Saving image: {}",
            output_path.to_path_buf().to_string_lossy()
        ));

        match save_image_format(&image, output_path, self.format.as_deref()) {
            Ok(()) => progress_bar.message("Image saved successfully"),
            Err(save_error) => {
                progress_bar.abort("Image failed to save");
                return Err(TaskError::SingleError(save_error).into());
            }
        }
        progress_bar.exit();
        Ok(())
    }
}
