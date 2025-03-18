mod command;
mod run;

use clap::Parser;
use std::error::Error;

use command::CommandArgs;

/// Fast, simple image manipulation tool
#[derive(Parser)]
#[command(version,about,long_about = None)]
pub struct Args {
    #[clap(flatten)]
    pub command_args: CommandArgs,
}

impl Args {
    pub fn run(self) -> Result<(), Box<dyn Error>> {
        self.command_args.run()?;
        Ok(())
    }
}
