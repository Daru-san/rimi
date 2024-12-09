use crate::{
    app,
    utils::{
        color::{BitDepth, ColorInfo, ColorSpace},
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
    #[clap(flatten)]
    color_args: ColorArgs,
}

#[derive(Parser)]
struct ColorArgs {
    /// Color space of the image
    #[clap(short, long, value_enum)]
    color_space: ColorSpace,

    /// Bit depth of the image
    #[clap(short = 'B', long, value_enum)]
    bit_depth: BitDepth,
}

impl RecolorArgs {
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

        let color_info = ColorInfo::new(
            self.color_args.color_space.clone(),
            self.color_args.bit_depth.clone(),
        );

        color_info.convert_image(&mut image);

        save_image_format(&image, output_path, None, app_args.overwrite)?;
        Ok(())
    }
}
