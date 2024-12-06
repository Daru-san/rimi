use clap::Parser;
mod batch;
mod completions;
mod convert;
mod info;
mod recolor;
mod resize;
mod transparent;

use batch::BatchArgs;
use completions::CompletionArgs;
use convert::ConvertArgs;
use info::InfoArgs;
use recolor::RecolorArgs;
use resize::ResizeArgs;
use transparent::TransparentArgs;

use std::error::Error;

#[derive(Parser)]
pub enum Command {
    /// Convert an image
    #[clap(short_flag = 'c')]
    Convert(ConvertArgs),

    /// Resize an image
    #[clap(short_flag = 'r')]
    Resize(ResizeArgs),

    /// Show image information
    #[clap(short_flag = 'i')]
    Info(InfoArgs),

    /// Batch image conversion
    #[clap(short_flag = 'b')]
    Batch(BatchArgs),

    /// Remove the background from an image
    #[clap(short_flag = 't')]
    Transparentize(TransparentArgs),

    /// Modify the image color type
    Recolor(RecolorArgs),

    /// Print shell completions
    Completions(CompletionArgs),
}

impl Command {
    pub fn run(self, app_args: &super::GlobalArgs) -> Result<(), Box<dyn Error>> {
        use Command::*;
        match self {
            Convert(args) => args.run(app_args),
            Resize(args) => args.run(app_args),
            Info(args) => args.run(),
            Batch(args) => args.run(app_args),
            Transparentize(args) => args.run(app_args),
            Recolor(args) => args.run(app_args),
            Completions(args) => args.run(),
        }
    }
}
