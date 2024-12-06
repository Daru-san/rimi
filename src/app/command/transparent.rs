use crate::{
    app,
    utils::image::{open_image, remove_background, save_image_format},
};
use clap::Parser;
use image::ImageFormat;
use std::error::Error;
use std::path::PathBuf;

#[derive(Parser)]
pub struct TransparentArgs {
    /// Path to the input image
    image_file: PathBuf,

    /// Path where the saved image should be written
    #[clap(short, long, global = true)]
    output: Option<String>,
}

impl TransparentArgs {
    pub fn run(&self, app_args: app::Args) -> Result<(), Box<dyn Error>> {
        let mut image = open_image(self.image_file.clone())?;

        let output_path = match &self.output {
            Some(e) => e.as_str(),
            None => self
                .image_file
                .as_os_str()
                .to_str()
                .expect("Error parsing image path: "),
        };
        if ImageFormat::from_path(self.image_file.clone()).unwrap() != ImageFormat::Png {
            return Err(format!(
                "{:?}: Image must be a png file.",
                self.image_file.as_os_str()
            )
            .into());
        }
        remove_background(&mut image);
        save_image_format(&image, output_path, None, app_args.overwrite)?;
        Ok(())
    }
}
