use std::mem::take;

use anyhow::{Error, Result};
use image::DynamicImage;

use crate::image::manipulator::convert_image;

use super::command::ImageCommand;

mod batch;
mod single;

pub trait RunSingle {
    fn run_single(&self, command: &ImageCommand, verbosity: u32) -> anyhow::Result<()>;
}

pub trait RunBatch {
    fn run_batch(&self, command: &ImageCommand, verbosity: u32) -> anyhow::Result<()>;
}

fn run_command(
    command: &ImageCommand,
    image: &mut DynamicImage,
    format: Option<&str>,
) -> Result<DynamicImage> {
    match command {
        ImageCommand::Convert => match convert_image(image, format) {
            Ok(mut image) => Ok(take(&mut image)),
            Err(convert_error) => Err(Error::msg(convert_error)),
        },
        ImageCommand::Resize(args) => match args.run(image) {
            Ok(()) => Ok(take(image)),
            Err(resize_error) => Err(resize_error),
        },
        ImageCommand::Recolor(args) => match args.run(image) {
            Ok(()) => Ok(take(image)),
            Err(recolor_error) => Err(recolor_error),
        },
        ImageCommand::Transparentize(args) => match args.run(image) {
            Ok(()) => Ok(take(image)),
            Err(removal_error) => Err(removal_error),
        },
    }
}

fn command_msg(command: &ImageCommand, image_name: &str) -> Result<String> {
    let message = match command {
        ImageCommand::Convert => "Converting",
        ImageCommand::Resize(_) => "Resizing",
        ImageCommand::Recolor(_) => "Recoloring",
        ImageCommand::Transparentize(_) => "Removing background",
    };
    Ok(format!("{message}: {image_name}"))
}
