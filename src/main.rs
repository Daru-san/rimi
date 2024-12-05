mod batch;
mod info;
mod open;
mod utils;

use std::path::PathBuf;
use batch::*;
use clap::{Parser, Subcommand};
use info::*;
use open::open_image;
use utils::*;

/// Simple image manipulation tool
#[derive(Parser, Debug)]
#[command(version,about,long_about = None)]
struct Args {
    #[command(subcommand)]
    cmd: Option<Commands>,

    /// Overwrite any existing files when saving the image
    #[clap(short = 'x', long, default_value = "false", global = true)]
    overwrite: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Convert an image
    #[clap(short_flag = 'c')]
    Convert {
        /// Format of the new image.
        #[clap(short, long)]
        format: Option<String>,
        /// Path to the input image
        image_file: PathBuf,

        /// Path where the saved image should be written
        #[clap(short, long, global = true)]
        output: Option<String>,
    },

    /// Resize an image
    #[clap(short_flag = 'r')]
    Resize {
        /// Path to the image file
        image_file: PathBuf,

        /// Path where the saved image should be written
        #[clap(short, long, global = true)]
        output: Option<String>,
        /// New width
        width: u32,

        /// New height
        height: u32,

        /// Image Sampling filter
        #[clap(short, long, default_value = "Nearest")]
        filter: String,

        /// Preserve aspect ratio
        #[clap(short, long)]
        preserve_aspect: bool,
    },

    /// Show image information
    #[clap(short_flag = 'i')]
    Info {
        #[clap(short, long)]
        short: bool,
        ///Path to the image file
        image_file: PathBuf,
    },

    /// Batch image conversion
    Batch {
        #[clap(value_parser,num_args = 1..100,value_delimiter = ' ')]
        images: Vec<String>,
    },
}

fn main() {
    let args = Args::parse();

    let do_overwrite = &args.overwrite;

    match &args.cmd {
        Some(Commands::Convert {
            format,
            image_file,
            output,
        }) => {
            let image = open_image(image_file.into());
            let output_path = match output {
                Some(e) => e.as_str(),
                None => image_file
                    .as_os_str()
                    .to_str()
                    .expect("Error parsing image path: "),
            };
            save_image_format(&image, output_path, format.as_deref(), *do_overwrite);
        }
        Some(Commands::Resize {
            width,
            height,
            filter,
            preserve_aspect,
            image_file,
            output,
        }) => {
            let mut image = open_image(image_file.into());
            let output_path = match output {
                Some(e) => e.as_str(),
                None => image_file
                    .as_os_str()
                    .to_str()
                    .expect("Error parsing image path: "),
            };
            resize_image(
                &mut image,
                Dimensions {
                    x: *width,
                    y: *height,
                },
                filter.to_string(),
                *preserve_aspect,
            );
            save_image_format(&image, output_path, None, *do_overwrite);
        }
        Some(Commands::Info { short, image_file }) => {
            let image = open_image(image_file.into());
            print_info(&image, image_file.to_path_buf(), *short);
        }
        Some(Commands::Batch { images }) => {
            let images_str = images.iter().map(|s| s.as_str()).collect();
            check_batch(images_str);
        }
        None => {
            println!("Please select one of: resize or convert.");
            println!("Use -h to get usage.")
        }
    }
}
