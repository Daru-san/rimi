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

use clap::{ArgGroup, CommandFactory, Parser, Subcommand};
use std::path::PathBuf;
use std::usize;

use anyhow::Result;

use crate::backend::error::AppError;

use super::run::{RunBatch, RunSingle};

#[derive(Parser)]
pub struct CommandArgs {
    #[clap(flatten)]
    misc_args: MiscArgs,

    /// Verbosity arguments
    #[clap(flatten)]
    verbosity_args: VerbosityArgs,

    /// Image manipulation arguments
    #[clap(flatten)]
    pub image_args: ImageArgs,
}

/// Image manipulation arguments
#[derive(Parser, Debug)]
#[clap(group = ArgGroup::new("image_group").conflicts_with("app_group"))]
pub struct ImageArgs {
    #[command(subcommand)]
    pub image_command: Option<ImageCommand>,

    /// Images to be converted
    #[clap(short,long,value_parser,num_args = 1..1000,value_delimiter = ' ',global = true)]
    pub images: Vec<PathBuf>,

    /// Output path, use a directory when batch converting, cannot be used with format
    #[clap(short, long, global = true)]
    pub output: Option<PathBuf>,

    /// Abort on error
    #[clap(short, long)]
    pub abort_on_error: bool,

    /// Overwrite existing images
    #[clap(short = 'x', long, global = true)]
    pub overwrite: bool,

    /// Output file name expression
    #[clap(short, long, global = true)]
    pub name_expr: Option<String>,

    /// Output image(s) format
    #[clap(short, long, global = true)]
    pub format: Option<String>,

    /// Tasks to run in parallel, spawns a thread for each Tasks
    #[clap(short, long)]
    pub tasks: usize,
}

#[derive(Parser)]
struct VerbosityArgs {
    /// Silence output
    #[clap(short, long, conflicts_with("verbose"))]
    quiet: bool,

    /// Very verbose output
    #[clap(short, long, conflicts_with("quiet"))]
    verbose: bool,
}

#[derive(Subcommand, Debug)]
pub enum ImageCommand {
    /// Convert an image
    #[clap(short_flag = 'c')]
    Convert,

    /// Resize an image
    #[clap(short_flag = 'r')]
    Resize(ResizeArgs),

    /// Remove the background from an image
    #[clap(short_flag = 't')]
    Transparentize(TransparentArgs),

    /// Modify the image color type
    Recolor(RecolorArgs),
}

#[derive(Parser, Debug)]
#[clap(group = ArgGroup::new("app_group").conflicts_with("image_group"))]
struct MiscArgs {
    #[command(subcommand)]
    pub command: Option<AppCommand>,
}

#[derive(Subcommand, Debug)]
pub enum AppCommand {
    /// Show image information
    Info(InfoArgs),

    /// Print shell completions
    Completions(CompletionArgs),
}

impl CommandArgs {
    pub fn run(&self) -> Result<()> {
        let verbosity = match (self.verbosity_args.quiet, self.verbosity_args.verbose) {
            (true, false) => 0,
            (false, true) => 2,
            (_, _) => 1,
        };

        match &self.misc_args.command {
            Some(AppCommand::Completions(args)) => args.run(),
            Some(AppCommand::Info(args)) => args.run(),
            None => match &self.image_args.image_command {
                Some(command) => match self.image_args.images.len() {
                    0 => Err(AppError::NoImages.into()),
                    1 => Ok(self.image_args.run_single(command, verbosity)?),
                    _ => {
                        Ok(self
                            .image_args
                            .run_batch(command, verbosity, self.image_args.tasks)?)
                    }
                },
                None => Ok(clap::Command::print_help(&mut super::Args::command())?),
            },
        }
    }
}
