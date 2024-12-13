mod completions;
mod info;
mod recolor;
mod resize;
mod transparent;

use completions::CompletionArgs;
use info::InfoArgs;
use recolor::RecolorArgs;
use resize::ResizeArgs;
use transparent::TransparentArgs;

use clap::Parser;
use std::error::Error;
use std::path::PathBuf;

use anyhow::Result;

use super::run::{RunBatch, RunSingle};

#[derive(Parser)]
pub struct CommandArgs {
    #[command(subcommand)]
    pub command: Command,

    /// Number of images to process in parallel
    #[clap(short, long, hide = true, default_value = "1", global = true)]
    pub parallel_images: u32,

    /// Images to be converted
    #[clap(short,long,value_parser,num_args = 1..1000,value_delimiter = ' ',required = false,global = true)]
    pub images: Vec<PathBuf>,

    /// Output path, use a directory when batch converting, cannot be used with format
    #[clap(short, long, global = true)]
    pub output: Option<PathBuf>,

    /// Abort on error
    #[clap(short, long)]
    pub abort_on_error: bool,

    #[clap(flatten)]
    pub extra_args: ExtraArgs,
}

#[derive(Parser, Debug)]
struct ExtraArgs {
    /// Overwrite existing images
    #[clap(short = 'x', long, global = true)]
    pub overwrite: bool,

    /// Output file name expression
    #[clap(short, long, global = true)]
    pub name_expr: Option<String>,

    /// Output image(s) format, cannot be used with output
    #[clap(short, long, global = true)]
    pub format: Option<String>,
}

#[derive(Parser, Debug)]
pub enum Command {
    /// Convert an image
    #[clap(short_flag = 'c')]
    Convert,

    /// Resize an image
    #[clap(short_flag = 'r')]
    Resize(ResizeArgs),

    /// Show image information
    Info(InfoArgs),

    /// Remove the background from an image
    #[clap(short_flag = 't')]
    Transparentize(TransparentArgs),

    /// Modify the image color type
    Recolor(RecolorArgs),

    /// Print shell completions
    Completions(CompletionArgs),
}

impl CommandArgs {
    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        match &self.command {
            Command::Completions(args) => args.run(),
            Command::Info(args) => args.run(),
            _ => match self.images.len() {
                0 => Err("Please add images".into()),
                1 => Ok(self.run_single()?),
                _ => Ok(self.run_batch()?),
            },
        }
    }
}
