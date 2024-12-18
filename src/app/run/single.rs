use anyhow::Error;

use super::RunSingle;
use crate::app::command::{ImageArgs, ImageCommand};
use crate::backend::error::TaskError;
use crate::backend::paths::prompt_overwrite_single;
use crate::backend::progress::AppProgress;
use crate::backend::progress::SingleProgress;
use crate::image::manipulator::{convert_image, open_image, save_image_format};

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

        match command {
            ImageCommand::Convert => match &self.format {
                Some(format) => {
                    progress.finish_task(&format!(
                        "Coverting image: {} to format {}",
                        image_path.to_path_buf().to_string_lossy(),
                        format
                    ));
                    image = match convert_image(&mut image, Some(format)) {
                        Ok(image) => image,
                        Err(e) => return Err(Error::msg(e)),
                    };
                }
                None => progress.finish_task(&format!(
                    "Converting image: {} as image {}",
                    image_path.to_path_buf().to_string_lossy(),
                    output_path.to_path_buf().to_string_lossy()
                )),
            },
            ImageCommand::Resize(args) => {
                progress.start_task("Resizing image");

                match args.run(&mut image) {
                    Ok(()) => {
                        progress.finish_task("Image resized successfully");
                    }
                    Err(resize_error) => {
                        progress.abort_task("Image resize failed with error.");
                        return Err(resize_error);
                    }
                }
            }
            ImageCommand::Recolor(args) => {
                progress.start_task("Recoloring image");
                match args.run(&mut image) {
                    Ok(()) => {
                        progress.finish_task("Image color changed.");
                    }
                    Err(recolor_error) => {
                        progress.abort_task("Image recolor failed with error.");
                        return Err(recolor_error);
                    }
                }
            }
            ImageCommand::Transparentize(args) => {
                progress.start_task("Removing image background");
                match args.run(&mut image) {
                    Ok(()) => progress.finish_task("Image background removed."),
                    Err(removal_error) => {
                        progress.abort_task("Background removal failed.");
                        return Err(removal_error);
                    }
                }
            }
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
