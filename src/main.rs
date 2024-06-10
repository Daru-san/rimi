use clap::Parser;
use image;

/// A simple image converter written in rust
#[derive(Parser, Debug)]
#[command(version,about,long_about = None)]
struct Args {
    /// File name of the source image
    #[arg(short, long)]
    source_image: String,

    /// File name of the final image after conversion with the file type
    #[arg(short, long)]
    final_image: String,
}

fn main() {
    let args = Args::parse();
    let img = image::open(args.source_image).unwrap();
    img.save(args.final_image).unwrap();
}
