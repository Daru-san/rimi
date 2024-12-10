use crate::app;
use crate::utils::image::{open_image, resize_image, save_image_format, Dimensions};
use clap::Parser;
use std::error::Error;
use std::path::PathBuf;

#[derive(Parser, Debug)]
pub struct ResizeArgs {
    /// Path to the image file
    image_file: PathBuf,

    /// Path where the saved image should be written
    #[clap(short, long, global = true)]
    output: Option<String>,

    /// New width
    width: u32,

    /// New height
    height: u32,

    /// Image Sampling filter
    #[clap(short, long, default_value = "Nearest")]
    filter: String,

    /// Preserve aspect ratio
    #[clap(short, long)]
    preserve_aspect: bool,
}

impl ResizeArgs {
    pub fn run(&self, app_args: &app::GlobalArgs) -> Result<(), Box<dyn Error>> {
        let mut image = open_image(self.image_file.clone())?;

        let output_path = match &self.output {
            Some(e) => e.as_str(),
            None => self
                .image_file
                .as_os_str()
                .to_str()
                .expect("Error parsing image path: "),
        };
        resize_image(
            &mut image,
            Dimensions {
                x: self.width,
                y: self.height,
            },
            self.filter.to_string(),
            self.preserve_aspect,
        )?;
        save_image_format(&image, output_path, None, app_args.overwrite)?;
        Ok(())
    }
}
