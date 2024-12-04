pub mod utils;
use std::path::Path;

use clap::{Parser, Subcommand};
use utils::*;

/// Simple image manipulation tool
#[derive(Parser, Debug)]
#[command(version,about,long_about = None)]
struct Args {
    #[command(subcommand)]
    cmd: Option<Commands>,

    /// Path to the input image
    filename: String,

    /// Path where the saved image should be written
    #[clap(short, long,requires_all = ["filename"])]
    output: Option<String>,

    /// Overwrite any existing files when saving the image
    #[clap(short = 'x', long, default_value = "false")]
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
    },

    /// Resize an image
    #[clap(short_flag = 'r')]
    Resize {
        /// New width
        #[clap(short)]
        x: u32,

        /// New height
        #[clap(short)]
        y: u32,

        /// Image Sampling filter
        #[clap(short, long, default_value = "Nearest")]
        filter: String,
    },
}

fn main() {
    let args = Args::parse();
    let (infile, outfile) = (&args.filename, &args.output);

    let do_overwrite = &args.overwrite;

    assert!(
        Path::new(infile).exists(),
        "File {} does not exist!",
        infile
    );

    let output_file = if outfile.is_some() {
        let result: String = outfile.clone().map(|s| s.to_string()).unwrap();
        result
    } else {
        infile.to_string()
    };

    let mut image = image::ImageReader::open(infile)
        .expect("Error opening image: ")
        .decode()
        .expect("Error decoding image: ");

    match &args.cmd {
        Some(Commands::Convert { format }) => {
            save_image_format(&image, &output_file, format.as_deref(), *do_overwrite);
        }
        Some(Commands::Resize { x, y, filter }) => {
            resize_image(&mut image, Dimensions { x: *x, y: *y }, filter.to_string());
            save_image_format(&image, &output_file, None, *do_overwrite);
        }
        None => {
            println!("Please select one of: resize or convert.");
            println!("Use -h to get usage.")
        }
    }
}
