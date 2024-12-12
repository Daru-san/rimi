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

use crate::app::state::{BatchErrors, TaskError, TaskQueue, TaskState};

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
    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        match &self.command {
            Command::Completions(args) => args.run(),
            Command::Info(args) => args.run(),
            _ => match self.images.len() {
                0 => Err("Please choose images".into()),
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

        let num_images = self.images.len() - 1;

        let mut tasks_queue = TaskQueue::new();
        for (index, image) in self.images.iter().enumerate() {
            let task_id = tasks_queue.new_task(image);
            let current_image = open_image(image.to_path_buf());
            match current_image {
                Ok(good_image) => {
                    tasks_queue.set_decoded(&good_image, task_id);
                }
                Err(error) => {
                    tasks_queue.set_failed(task_id, error);
                }
            }
        }

        if tasks_queue.has_failures() {
            if self.abort_on_error {
                let mut total_errors = Vec::new();
                for task in tasks_queue.failed_tasks().iter() {
                    if let TaskState::Failed(error) = &task.state {
                        total_errors.push(TaskError(error.0.clone()));
                    }
                }

                return Err(Box::new(BatchErrors(total_errors)));
            }
        } else {
        }

        let output_path = match &self.output {
            Some(path) => path.to_path_buf(),
            None => PathBuf::from("."),
        };

        let mut decoded_paths = Vec::new();
        for task in tasks_queue.decoded_tasks().iter() {
            decoded_paths.push(task.image_path.to_path_buf());
        }

        let out_paths = create_paths(
            decoded_paths,
            output_path,
            self.extra_args.name_expr.as_deref(),
        )?;

        for (index, path) in out_paths.iter().enumerate() {
            tasks_queue.set_out_path(tasks_queue.decoded_ids()[index], path);
        }
        let count = tasks_queue.decoded_tasks().len();

        for index in 0..count {
            let task_id = tasks_queue.decoded_tasks()[0].id;

            let count = count - 1;

            let mut current_task = {
                match tasks_queue.task_by_id_mut(task_id) {
                    Some(task) => task.clone(),
                    _ => return Err("No such task".into()),
                }
            };

            match &self.command {
                Command::Convert => (),
                Command::Resize(args) => {
                    match args.run(&mut current_task.image) {
                        Ok(()) => {
                            tasks_queue.set_processed(&current_task.image, task_id);
                        }
                        Err(e) => {
                            tasks_queue.set_failed(task_id, e.to_string());
                        }
                    }
                }
                Command::Recolor(args) => {
                    match args.run(&mut current_task.image) {
                        Ok(()) => {
                            tasks_queue.set_processed(&current_task.image, task_id);
                        }
                        Err(e) => {
                            tasks_queue.set_failed(task_id, e.to_string());
                        }
                    }
                }
                Command::Transparentize(args) => {
                    match args.run(&mut current_task.image) {
                        Ok(()) => {
                            tasks_queue.set_processed(&current_task.image, task_id);
                        }
                        Err(e) => {
                            tasks_queue.set_failed(task_id, e.to_string());
                        }
                    }
                }
                command => {
                    return Err(format!("{:?} cannot be run right now", command).into());
                }
            };
            if let Some(task) = tasks_queue.task_by_id(task_id) {
                if let TaskState::Failed(_) = task.state {
                    continue;
                }
            }
            let image_result = save_image_format(
                &current_task.image,
                &current_task.out_path,
                self.extra_args.format.as_deref(),
                self.extra_args.overwrite,
            );

            match image_result {
                Ok(()) => {
                    tasks_queue.set_completed(task_id);
                }
                Err(e) => {
                    tasks_queue.set_failed(task_id, e.to_string());
                }
            }
        }
        Ok(())
    }
}
