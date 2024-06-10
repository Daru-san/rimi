use clap::Parser;
use std::process::Command;

#[derive(Parser, Debug)]
#[command(version,about,long_about = None)]
struct Cli {
    #[arg(short, long)]
    source_image: String,

    #[arg(short, long)]
    final_image: String,
}
fn main() {
    let args = Cli::parse();

    Command::new("convert")
        .arg(args.source_image)
        .arg(args.final_image)
        .spawn()
        .expect("Image conversion failed, check if `convert` is in your PATH");
}
