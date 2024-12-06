use crate::{
    app,
    utils::{
        color::ColorInfo,
        image::{open_image, save_image_format},
    },
};
use clap::Parser;
use std::error::Error;
use std::path::PathBuf;

#[derive(Parser)]
pub struct RecolorArgs {
    /// Path to the input image
    image_file: PathBuf,

    /// Path where the saved image should be written
    #[clap(short, long, global = true)]
    output: Option<String>,

    /// Color type
    #[clap(short, long)]
    color_type: ColorInfo,
}

impl RecolorArgs {
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

        self.color_type.convert_image(&mut image);

        save_image_format(&image, output_path, None, app_args.overwrite)?;
        Ok(())
    }
}
