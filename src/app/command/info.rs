use crate::backend::error::TaskError;
use crate::{utils::image::open_image, utils::info::print_info};
use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
pub struct InfoArgs {
    /// Shorted information
    #[clap(short, long)]
    short: bool,

    ///Path to the image file
    image_file: PathBuf,
}

impl InfoArgs {
    pub fn run(&self) -> Result<()> {
        let image = match open_image(&self.image_file) {
            Ok(image) => image,
            Err(decode_error) => return Err(TaskError::SingleError(decode_error).into()),
        };
        print_info(&image, self.image_file.to_path_buf(), self.short);
        Ok(())
    }
}
