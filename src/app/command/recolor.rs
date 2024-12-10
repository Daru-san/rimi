use crate::utils::color::ColorInfo;
use clap::Parser;
use image::DynamicImage;
use std::error::Error;

#[derive(Parser, Debug)]
pub struct RecolorArgs {
    /// Color type
    #[clap(short, long)]
    color_type: ColorInfo,
}

impl RecolorArgs {
    pub fn run(&self, image: &mut DynamicImage) -> Result<(), Box<dyn Error>> {
        self.color_type.convert_image(image);
        Ok(())
    }
}
