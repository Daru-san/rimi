use crate::backend::error::TaskError;
use crate::image::manipulator::{resize_image, Dimensions};

use anyhow::Result;
use clap::Parser;
use image::DynamicImage;

#[derive(Parser, Debug)]
pub struct ResizeArgs {
    /// New width
    width: u32,

    /// New height
    height: u32,

    /// Image Sampling filter
    #[clap(short = 'F', long, default_value = "Nearest")]
    filter: String,

    /// Preserve aspect ratio
    #[clap(short = 'P', long)]
    preserve_aspect: bool,
}

impl ResizeArgs {
    pub fn run(&self, image: &mut DynamicImage) -> Result<()> {
        match resize_image(
            image,
            Dimensions {
                x: self.width,
                y: self.height,
            },
            self.filter.to_string(),
            self.preserve_aspect,
        ) {
            Ok(()) => Ok(()),
            Err(resize_error) => Err(TaskError::SingleError(resize_error).into()),
        }
    }
}
