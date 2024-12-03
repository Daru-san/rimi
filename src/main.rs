pub mod utils;
use std::path::Path;

use clap::{Parser, Subcommand};
use utils::*;

/// Simple in-development image manipulation tool
#[derive(Parser, Debug)]
#[command(version,about,long_about = None)]
struct Args {
    #[command(subcommand)]
    cmd: Option<Commands>,

    /// Input image filename
    filename: String,

    /// Output image
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
        /// Image format
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

        #[clap(short, long, default_value = "Nearest")]
        r#type: String,
    },
}

fn main() {
    let args = Args::parse();
    let (infile, outfile) = (&args.filename, &args.output);

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

    let mut image = image::open(infile).expect("Error parsing image file:");

    match &args.cmd {
        Some(Commands::Convert { format }) => {
            if !format.is_none() {
                save_image_format(image, &output_file, format.clone());
            } else {
                save_image_format(image, &output_file, None);
            }
        }
        Some(Commands::Resize { x, y, r#type }) => {
            resize_image(&mut image, Dimensions { x: *x, y: *y }, r#type.to_string());
            save_image(&image, &output_file);
        }
        None => {
            println!("Please choose a command");
        }
    }
}
