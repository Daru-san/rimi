use crate::{utils::image::open_image, utils::info::print_info};
use clap::Parser;
use std::error::Error;
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
    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        let image = open_image(self.image_file.clone())?;
        print_info(&image, self.image_file.to_path_buf(), self.short);
        Ok(())
    }
}
