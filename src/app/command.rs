mod completions;
mod info;
mod recolor;
mod resize;
mod transparent;

use completions::CompletionArgs;
use image::DynamicImage;
use info::InfoArgs;
use recolor::RecolorArgs;
use resize::ResizeArgs;
use transparent::TransparentArgs;

use clap::Parser;
use std::error::Error;
use std::path::PathBuf;

use crate::app::state::AppState;

#[derive(Parser)]
pub struct CommandArgs {
    #[command(subcommand)]
    pub command: Command,

    /// Number of images to process in parallel
    #[clap(short, long, hide = true, default_value = "1")]
    parallel_images: u32,

    /// Images to be converted
    #[clap(value_parser,num_args = 1..1000,value_delimiter = ' ',required = true)]
    images: Vec<PathBuf>,

    /// Output path, use a directory when batch converting, cannot be used with format
    #[clap(short, long)]
    output: Option<PathBuf>,

    /// Abort on error
    #[clap(short, long)]
    abort_on_error: bool,

    #[clap(flatten)]
    extra_args: ExtraArgs,
}

#[derive(Parser, Debug)]
struct ExtraArgs {
    /// Overwrite existing images
    #[clap(short, long)]
    overwrite: bool,

    /// Output file name expression
    name_expr: Option<String>,

    /// Output image(s) format, cannot be used with output
    #[clap(short, long)]
    format: Option<String>,
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
    pub fn run(self, app_args: &super::GlobalArgs) -> Result<(), Box<dyn Error>> {
        if self.extra_args.format.is_some() && self.output.is_some() {
            return Err("Select either format or output".into());
        }
        match self.command {
            Command::Completions(args) => args.run(),
            Command::Info(args) => args.run(),
            _ => match self.images.len() {
                1 => self.run_single(app_args),
                _ => self.run_batch(app_args),
            },
        }
    }

    fn run_single(self, app_args: &super::GlobalArgs) -> Result<(), Box<dyn Error>> {
        use crate::utils::image::{open_image, save_image_format};
        let image_path = &self.images[0];
        let mut image = open_image(image_path.to_path_buf())?;

        let output_path = match &self.output {
            Some(path) => path,
            None => &image_path,
        };
        match self.command {
            Command::Convert => ()
            Command::Resize(args) => args.run(&mut image)?,
            Command::Recolor(args) => args.run(&mut image)?,
            Command::Transparentize(args) => args.run(&mut image, image_path)?,
            command => {
                return Err(format!("{:?} cannot be run with the batch flag", command).into());
            }
        };
        save_image_format(
            &image,
            output_path,
            self.extra_args.format.as_deref(),
            self.extra_args.overwrite
        )?;
        Ok(())
    }
}
