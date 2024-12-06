use crate::utils::image::{open_image, save_image_format};
use clap::Parser;
use std::error::Error;
use std::path::PathBuf;

#[derive(Parser)]
pub struct ConvertArgs {
    /// Format of the new image.
    #[clap(short, long)]
    format: Option<String>,

    /// Path to the input image
    image_file: PathBuf,

    /// Path where the saved image should be written
    #[clap(short, long, global = true)]
    output: Option<String>,
}

impl ConvertArgs {
    pub fn run(&self, app_args: &crate::app::GlobalArgs) -> Result<(), Box<dyn Error>> {
        let image = open_image(self.image_file.clone())?;

        let output_path = match &self.output {
            Some(e) => e.as_str(),
            None => self
                .image_file
                .as_os_str()
                .to_str()
                .expect("Error parsing image path: "),
        };
        save_image_format(
            &image,
            output_path,
            self.format.as_deref(),
            app_args.overwrite,
        )?;
        Ok(())
    }
}
