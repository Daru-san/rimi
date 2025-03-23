use crate::image::color::{BitDepth, ColorInfo, ColorSpace};

use anyhow::Result;

use clap::Parser;
use image::DynamicImage;

#[derive(Parser, Debug, Clone)]
pub struct RecolorArgs {
    /// Color space of the image
    #[clap(short, long, value_enum)]
    color_space: ColorSpace,

    /// Bit depth of the image
    #[clap(short, long, value_enum)]
    bit_depth: BitDepth,
}

impl RecolorArgs {
    pub fn run(&self, image: DynamicImage) -> Result<DynamicImage> {
        let color_info = ColorInfo::new(&self.color_space, &self.bit_depth);

        Ok(color_info.convert_image(image))
    }
}
