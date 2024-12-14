use crate::image::color::ColorInfo;

use anyhow::Result;
use clap::Parser;
use image::DynamicImage;

#[derive(Parser, Debug)]
pub struct RecolorArgs {
    /// Color type
    #[clap(short, long)]
    color_type: ColorInfo,
}

impl RecolorArgs {
    pub fn run(&self, image: &mut DynamicImage) -> Result<()> {
        self.color_type.convert_image(image);
        Ok(())
    }
}
