use crate::utils::image::remove_background;
use clap::Parser;
use image::{DynamicImage, ImageFormat};
use std::error::Error;
use std::path::PathBuf;

#[derive(Parser, Debug)]
pub struct TransparentArgs {}

impl TransparentArgs {
    pub fn run(
        &self,
        image: &mut DynamicImage,
        image_path: &PathBuf,
    ) -> Result<(), Box<dyn Error>> {
        if ImageFormat::from_path(image_path).unwrap() != ImageFormat::Png {
            return Err(format!("{:?}: Image must be a png file.", image_path.as_os_str()).into());
        }
        remove_background(image);
        Ok(())
    }
}
