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
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Convert an image
    Convert {
        /// Image format
        #[clap(short, long)]
        format: Option<String>,
    },

    /// Resize an image
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

    let mut image = Image::new(infile.to_string());
    if !outfile.is_none() {
        image.outpath = outfile
            .clone()
            .map(|s| s.to_string())
            .expect("Unexpected error occured");
    }

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
