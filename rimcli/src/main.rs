mod app;
mod backend;
mod image;

use std::error::Error;

use app::Args;
use clap::Parser;
use std::process::exit;

fn main() {
    match try_run() {
        Ok(()) => {
            exit(0);
        }
        Err(e) => {
            eprintln!("{}", e);
            exit(0);
        }
    }
}

fn try_run() -> Result<(), Box<dyn Error>> {
    Args::parse().run()
}
