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
        let image = match open_image(self.image_file.clone()) {
            Ok(image) => image,
            Err(e) => return Err(TaskError::SingleError(e).into()),
        };
        print_info(&image, self.image_file.to_path_buf(), self.short);
        Ok(())
    }
}
