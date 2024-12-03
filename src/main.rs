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
        #[clap(short, long)]
        width: u32,

        /// New height
        #[clap(short, long)]
        height: u32,
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

    let mut image = image::open(infile).expect("Something happened");
    match &args.cmd {
        Some(Commands::Convert { format }) => {
            if !format.is_none() {
                image.format = format
                    .clone()
                    .map(|s| s.to_string())
                    .expect("Unexpected error occured");
            }
            image.convert();
        }
        Some(Commands::Resize { width, height }) => {
            image.resize_dim(*width, *height);
            image.convert();
        }
        None => {
            println!("Please choose a command");
        }
    }
}
