use crate::image::manipulator::remove_background;

use anyhow::Result;
use clap::Parser;
use image::DynamicImage;

#[derive(Parser, Debug)]
pub struct TransparentArgs {}

impl TransparentArgs {
    pub fn run(&self, image: &mut DynamicImage) -> Result<()> {
        remove_background(image);
        Ok(())
    }
}
