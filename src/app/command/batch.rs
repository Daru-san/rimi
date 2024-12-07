use crate::utils::batch::*;
use crate::utils::image::save_image_format;
use clap::Parser;
use image::ImageReader;
use std::error::Error;

#[derive(Parser)]
pub struct BatchArgs {
    /// Images to be converted
    #[clap(value_parser,num_args = 1..100,value_delimiter = ' ',required = true)]
    images: Vec<String>,

    /// Optional output directory where all output images will be saved
    #[clap(short, long, default_value = ".")]
    directory: String,

    /// Expression that the output image names will follow
    #[clap(short, long)]
    name_expr: Option<String>,
}

//TODO: Add progress indicators
impl BatchArgs {
    pub fn run(&self, app_args: &crate::app::GlobalArgs) -> Result<(), Box<dyn Error>> {
        let images_str: Vec<&str> = self.images.iter().map(|s| s.as_str()).collect();

        match check_batch(images_str.clone()) {
            Ok(_) => (),
            Err(errors) => {
                let mut err_string = String::new();
                for error in &errors {
                    err_string.push_str(error);
                }
                return Err(Box::new(BatchError(errors)));
            }
        }

        let paths = create_paths(
            images_str.clone(),
            self.directory.as_str(),
            self.name_expr.as_deref(),
        )?;

        for (index, image_str) in self.images.iter().enumerate() {
            let image = ImageReader::open(image_str).unwrap().decode()?;
            save_image_format(&image, &paths[index], None, app_args.overwrite)?;
        }
        Ok(())
    }
}
