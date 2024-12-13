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

        single_progress.start_operation(
            format!(
                "Decoding image: {}",
                image_path.to_path_buf().to_string_lossy()
            )
            .as_str(),
        );

        let mut image = match open_image(image_path.to_path_buf()) {
            Ok(image) => {
                single_progress.complete_operation_with_message("Image decoded successfully");
                image
            }
            Err(e) => {
                single_progress.abort_message("Image decode failed");
                return Err(TaskError::SingleError(e).into());
            }
        };

        let output_path = match &self.output {
            Some(path) => path,
            None => image_path,
        };

        single_progress.complete_operation_with_message(
            format!(
                "Set output path: {}",
                output_path.to_path_buf().to_string_lossy()
            )
            .as_str(),
        );

        match command {
            ImageCommand::Convert => match &self.format {
                Some(format) => single_progress.complete_operation_with_message(
                    format!(
                        "Coverting image: {} to format {}",
                        image_path.to_path_buf().to_string_lossy(),
                        format
                    )
                    .as_str(),
                ),
                None => single_progress.complete_operation_with_message(
                    format!(
                        "Converting image: {} as image {}",
                        image_path.to_path_buf().to_string_lossy(),
                        output_path.to_path_buf().to_string_lossy()
                    )
                    .as_str(),
                ),
            },
            ImageCommand::Resize(args) => {
                single_progress.start_operation("Resizing image");

                match args.run(&mut image) {
                    Ok(()) => {
                        single_progress
                            .complete_operation_with_message("Image resized successfully");
                    }
                    Err(e) => {
                        single_progress.abort_message("Image resize failed with error.");
                        return Err(e);
                    }
                }
            }
            ImageCommand::Recolor(args) => {
                single_progress.start_operation("Recoloring image");
                match args.run(&mut image) {
                    Ok(()) => {
                        single_progress.complete_operation_with_message("Image color changed.");
                    }
                    Err(e) => {
                        single_progress.abort_message("Image recolor failed with error.");
                        return Err(e);
                    }
                }
            }
            ImageCommand::Transparentize(args) => {
                single_progress.start_operation("Removing image background");
                match args.run(&mut image) {
                    Ok(()) => {
                        single_progress.complete_operation_with_message("Image background removed.")
                    }
                    Err(e) => {
                        single_progress.abort_message("Background removal failed.");
                        return Err(e);
                    }
                }
            }
        };
        single_progress.start_operation(
            format!(
                "Saving image: {}",
                output_path.to_path_buf().to_string_lossy()
            )
            .as_str(),
        );

        match save_image_format(&image, output_path, self.format.as_deref(), self.overwrite) {
            Ok(()) => single_progress.complete_operation_with_message("Image saved successfully"),
            Err(e) => {
                single_progress.abort_message("Image failed to save");
                return Err(TaskError::SingleError(e).into());
            }
        }
        single_progress.complete();
        Ok(())
    }
}
