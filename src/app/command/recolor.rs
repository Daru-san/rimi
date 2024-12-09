use crate::image::color::{BitDepth, ColorInfo, ColorSpace};

use anyhow::Result;

use clap::Parser;
use image::DynamicImage;

#[derive(Parser, Debug)]
pub struct RecolorArgs {
    /// Color space of the image
    #[clap(short, long, value_enum)]
    color_space: ColorSpace,

    /// Bit depth of the image
    #[clap(short = 'B', long, value_enum)]
    bit_depth: BitDepth,
}

impl RecolorArgs {
    pub fn run(&self, image: &mut DynamicImage) -> Result<()> {
        let color_info = ColorInfo::new(&self.color_space, &self.bit_depth);

        color_info.convert_image(image);
        Ok(())
    }
}
