use crate::utils::image::remove_background;
use clap::Parser;
use image::DynamicImage;
use std::error::Error;

#[derive(Parser, Debug)]
pub struct TransparentArgs {}

impl TransparentArgs {
    pub fn run(&self, image: &mut DynamicImage) -> Result<(), Box<dyn Error>> {
        remove_background(image);
        Ok(())
    }
}
