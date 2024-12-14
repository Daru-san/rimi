use super::RunSingle;
use crate::app::command::{ImageArgs, ImageCommand};
use crate::backend::error::TaskError;
use crate::backend::progress::AppProgress;
use crate::backend::progress::SingleProgress;
use console::Style;

impl RunSingle for ImageArgs {
    fn run_single(&self, command: &ImageCommand, verbosity: u32) -> anyhow::Result<()> {
        use crate::utils::image::{open_image, save_image_format};

        let image_path = &self.images[0];

        let style = Style::new().blue().bold().underlined();

        let msg = style.apply_to("Starting 4 tasks");

        println!("{}", msg);

        let single_progress = SingleProgress::init(verbosity);

        single_progress.start_task(
            format!(
                "Decoding image: {}",
                image_path.to_path_buf().to_string_lossy()
            )
            .as_str(),
        );

        let mut image = match open_image(image_path) {
            Ok(image) => {
                single_progress.finish_task("Image decoded successfully");
                image
            }
            Err(decode_error) => {
                single_progress.abort_task("Image decode failed");
                return Err(TaskError::SingleError(decode_error).into());
            }
        };

        let output_path = match &self.output {
            Some(path) => path,
            None => image_path,
        };

        single_progress.finish_task(
            format!(
                "Set output path: {}",
                output_path.to_path_buf().to_string_lossy()
            )
            .as_str(),
        );

        match command {
            ImageCommand::Convert => match &self.format {
                Some(format) => single_progress.finish_task(
                    format!(
                        "Coverting image: {} to format {}",
                        image_path.to_path_buf().to_string_lossy(),
                        format
                    )
                    .as_str(),
                ),
                None => single_progress.finish_task(
                    format!(
                        "Converting image: {} as image {}",
                        image_path.to_path_buf().to_string_lossy(),
                        output_path.to_path_buf().to_string_lossy()
                    )
                    .as_str(),
                ),
            },
            ImageCommand::Resize(args) => {
                single_progress.start_task("Resizing image");

                match args.run(&mut image) {
                    Ok(()) => {
                        single_progress.finish_task("Image resized successfully");
                    }
                    Err(resize_error) => {
                        single_progress.abort_task("Image resize failed with error.");
                        return Err(resize_error);
                    }
                }
            }
            ImageCommand::Recolor(args) => {
                single_progress.start_task("Recoloring image");
                match args.run(&mut image) {
                    Ok(()) => {
                        single_progress.finish_task("Image color changed.");
                    }
                    Err(recolor_error) => {
                        single_progress.abort_task("Image recolor failed with error.");
                        return Err(recolor_error);
                    }
                }
            }
            ImageCommand::Transparentize(args) => {
                single_progress.start_task("Removing image background");
                match args.run(&mut image) {
                    Ok(()) => single_progress.finish_task("Image background removed."),
                    Err(removal_error) => {
                        single_progress.abort_task("Background removal failed.");
                        return Err(removal_error);
                    }
                }
            }
        };
        single_progress.start_task(
            format!(
                "Saving image: {}",
                output_path.to_path_buf().to_string_lossy()
            )
            .as_str(),
        );

        match save_image_format(&image, output_path, self.format.as_deref(), self.overwrite) {
            Ok(()) => single_progress.finish_task("Image saved successfully"),
            Err(save_error) => {
                single_progress.abort_task("Image failed to save");
                return Err(TaskError::SingleError(save_error).into());
            }
        }
        single_progress.exit();
        Ok(())
    }
}
