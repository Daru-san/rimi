use clap::Parser;
use image;

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
}
