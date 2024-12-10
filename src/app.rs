mod command;
mod state;

use clap::Parser;
use std::error::Error;

use command::Command;

/// Simple image manipulation tool
#[derive(Parser)]
#[command(version,about,long_about = None)]
pub struct Args {
    #[clap(flatten)]
    pub command_args: CommandArgs,

    #[clap(flatten)]
    global_args: GlobalArgs,
}

#[derive(Parser)]
pub struct GlobalArgs {
    /// Overwrite any existing files when saving the image
    #[clap(short = 'x', long, default_value = "false", global = true)]
    pub overwrite: bool,
}

impl Args {
    pub fn run(self) -> Result<(), Box<dyn Error>> {
        self.command_args.run(&self.global_args);
        Ok(())
    }
}
