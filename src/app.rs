mod command;
mod state;

use clap::Parser;
use std::error::Error;

use command::CommandArgs;

/// Simple image manipulation tool
#[derive(Parser)]
#[command(version,about,long_about = None)]
pub struct Args {
    #[clap(flatten)]
    pub command_args: CommandArgs,
}

impl Args {
    pub fn run(self) -> Result<(), Box<dyn Error>> {
        self.command_args.run(&self.global_args);
        Ok(())
    }
}
