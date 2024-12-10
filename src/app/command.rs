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
    #[clap(short, long, hide = true, default_value = "1", global = true)]
    parallel_images: u32,

    /// Images to be converted
    #[clap(short,long,value_parser,num_args = 1..1000,value_delimiter = ' ',required = false,global = true)]
    images: Vec<PathBuf>,

    /// Output path, use a directory when batch converting, cannot be used with format
    #[clap(short, long, global = true)]
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
    #[clap(short = 'x', long, global = true)]
    overwrite: bool,

    /// Output file name expression
    #[clap(short, long, global = true)]
    name_expr: Option<String>,

    /// Output image(s) format, cannot be used with output
    #[clap(short, long, global = true)]
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
    pub fn run(self) -> Result<(), Box<dyn Error>> {
        if self.extra_args.format.is_some() && self.output.is_some() {
            return Err("Select either format or output".into());
        }
        match self.command {
            Command::Completions(args) => args.run(),
            Command::Info(args) => args.run(),
            _ => match self.images.len() {
                1 => self.run_single(),
                _ => self.run_batch(),
            },
        }
    }

    fn run_single(&self) -> Result<(), Box<dyn Error>> {
        use crate::utils::image::{open_image, save_image_format};
        let image_path = &self.images[0];
        let mut image = open_image(image_path.to_path_buf())?;

        let output_path = match &self.output {
            Some(path) => path,
            None => image_path,
        };
        match &self.command {
            Command::Convert => (),
            Command::Resize(args) => args.run(&mut image)?,
            Command::Recolor(args) => args.run(&mut image)?,
            Command::Transparentize(args) => args.run(&mut image)?,
            _ => {}
        };
        save_image_format(
            &image,
            output_path,
            self.extra_args.format.as_deref(),
            self.extra_args.overwrite,
        )?;
        Ok(())
    }

    fn run_batch(&self) -> Result<(), Box<dyn Error>> {
        use crate::utils::batch::*;
        use crate::utils::image::{open_image, save_image_format};

        let mut good_images: Vec<DynamicImage> = Vec::new();
        let mut image_errors = Vec::new();
        let mut paths = Vec::new();

        for image in &self.images {
            let current_image = open_image(image.clone());
            match current_image {
                Ok(good_image) => {
                    good_images.push(good_image);
                    paths.push(image.to_path_buf());
                }
                Err(e) => image_errors.push(e),
            }
        }

        if self.abort_on_error {
            return Err(Box::new(BatchError(image_errors)));
        }

        let output_path = match &self.output {
            Some(path) => path.to_path_buf(),
            None => PathBuf::from("."),
        };

        let out_paths = create_paths(paths, output_path, self.extra_args.name_expr.as_deref())?;

        for (index, mut image) in good_images.iter_mut().enumerate() {
            match &self.command {
                Command::Convert => (),
                Command::Resize(args) => args.run(&mut image)?,
                Command::Recolor(args) => args.run(&mut image)?,
                Command::Transparentize(args) => args.run(&mut image)?,
                command => {
                    return Err(format!("{:?} cannot be run with the batch flag", command).into());
                }
            };
            save_image_format(image, &out_paths[index], None, self.extra_args.overwrite)?;
        }
        Ok(())
    }
}
