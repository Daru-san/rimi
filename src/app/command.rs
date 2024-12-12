mod completions;
mod info;
mod recolor;
mod resize;
mod transparent;

use completions::CompletionArgs;
use console::Style;
use indicatif::{ProgressBar, ProgressStyle};
use info::InfoArgs;
use recolor::RecolorArgs;
use resize::ResizeArgs;
use transparent::TransparentArgs;

use clap::Parser;
use std::error::Error;
use std::path::PathBuf;
use std::time::Duration;

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
        let style = Style::new().blue().bold().underlined();

        let msg = style.apply_to("Starting 4 tasks");
        println!("{}", msg);

        let progress = ProgressBar::new(4).with_style(
            ProgressStyle::with_template("[{pos}/{len}] {msg} {bar:40.cyan/blue} [{duration}]")
                .unwrap()
                .progress_chars("##-"),
        );

        progress.enable_steady_tick(Duration::from_millis(100));

        progress.set_message(format!(
            "Decoding image: {}",
            image_path.to_path_buf().to_string_lossy()
        ));

        let mut image = open_image(image_path.to_path_buf())?;
        progress.inc(1);
        progress.println(format!(
            "[{}/4] Image decoded successfully",
            progress.position()
        ));

        let output_path = match &self.output {
            Some(path) => path,
            None => image_path,
        };

        progress.inc(1);
        progress.println(format!(
            "[{}/{}] Set output path: {}",
            progress.position(),
            4,
            output_path.to_path_buf().to_string_lossy()
        ));

        match &self.command {
            Command::Convert => match &self.extra_args.format {
                Some(format) => progress.println(format!(
                    "[{}/4] Coverting image: {} to format {}",
                    progress.position(),
                    image_path.to_path_buf().to_string_lossy(),
                    format
                )),
                None => progress.println(format!(
                    "[{}/4] Converting image: {} as image {}",
                    progress.position(),
                    image_path.to_path_buf().to_string_lossy(),
                    output_path.to_path_buf().to_string_lossy()
                )),
            },
            Command::Resize(args) => {
                progress.set_message("Resizing image");
                args.run(&mut image)?;
                progress.println(format!(
                    "[{}/4] Image resized successfully!",
                    progress.position()
                ));
            }
            Command::Recolor(args) => {
                progress.set_message("Recoloring image");
                args.run(&mut image)?;
                progress.println(format!(
                    "[{}/4] Image recolored successfully",
                    progress.position()
                ));
            }
            Command::Transparentize(args) => {
                progress.set_message("Removing background");
                args.run(&mut image)?;
                progress.println(format!("[{}/4] Background removed", progress.position()));
            }
            _ => {}
        };
        progress.inc(0x1);
        progress.set_message(format!(
            "Saving image: {}",
            output_path.to_path_buf().to_string_lossy()
        ));

        save_image_format(
            &image,
            output_path,
            self.extra_args.format.as_deref(),
            self.extra_args.overwrite,
        )?;
        progress.println(format!(
            "[{}/4] Image saved successfully",
            progress.position()
        ));
        progress.finish_with_message("All image operations complete!");
        Ok(())
    }

    fn run_batch(&self) -> Result<(), Box<dyn Error>> {
        use crate::utils::batch::*;
        use crate::utils::image::{open_image, save_image_format};

        let num_images = self.images.len() - 1;

        let style = Style::new().bold().blue();

        let msg = style.apply_to(format!("Trying to decode {} images", self.images.len()));

        println!("{}", msg);

        let mut tasks_queue = TaskQueue::new();

        let progress = ProgressBar::new_spinner();
        progress.enable_steady_tick(Duration::from_millis(100));
        for (index, image) in self.images.iter().enumerate() {
            let task_id = tasks_queue.new_task(image);

            progress.set_message(format!(
                "[{}/{}] Decoding image: {:?}",
                index,
                num_images,
                image.file_name().as_slice()
            ));

            progress.inc(1);

            let current_image = open_image(image.to_path_buf());

            match current_image {
                Ok(good_image) => {
                    tasks_queue.set_decoded(&good_image, task_id);
                }
                Err(error) => {
                    progress.println(format!(
                        "[{}/{}] Failed to decode image: {:?}",
                        index,
                        num_images,
                        image.file_name().as_slice()
                    ));
                    tasks_queue.set_failed(task_id, error);
                }
            }
        }

        if tasks_queue.has_failures() {
            progress.finish_with_message(format!(
                "{} images were decoded with {} errors",
                num_images,
                tasks_queue.count_failures()
            ));
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
            progress.finish_with_message(format!(
                "{} images were decoded successfully ^.^",
                tasks_queue.decoded_tasks().len()
            ));
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

        let msg = style.apply_to(format!(
            "Processing {} images",
            tasks_queue.decoded_tasks().len()
        ));

        println!("{}", msg);

        let progress = ProgressBar::new_spinner();
        progress.enable_steady_tick(Duration::from_millis(100));

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
                    progress.set_message("Resizing image");
                    match args.run(&mut current_task.image) {
                        Ok(()) => {
                            tasks_queue.set_processed(&current_task.image, task_id);
                            progress
                                .println(format!("[{index}/{count}] Image resized successfully"));
                        }
                        Err(e) => {
                            tasks_queue.set_failed(task_id, e.to_string());
                            progress.println("Image resize exited with error.");
                        }
                    }
                }
                Command::Recolor(args) => {
                    progress.println(format!("[{}/{}] Recoloring image", index, count,));
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
                    progress.println(format!("[{}/{}] Removing background", index, count,));
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

            progress.set_message(format!(
                "[{index}/{count}] Saving image: {:?}",
                &current_task.out_path.file_name().as_slice()
            ));

            let image_result = save_image_format(
                &current_task.image,
                &current_task.out_path,
                self.extra_args.format.as_deref(),
                self.extra_args.overwrite,
            );

            match image_result {
                Ok(()) => {
                    tasks_queue.set_completed(task_id);
                    progress.println(format!(
                        "[{index}/{count}] Image processing complete: {:?}",
                        &current_task.out_path.file_name().as_slice()
                    ));
                }
                Err(e) => {
                    tasks_queue.set_failed(task_id, e.to_string());
                    progress.println(format!(
                        "[{index}/{count}] Image processing failed with error: {:?}",
                        &current_task.out_path.file_name().as_slice()
                    ));
                }
            }
        }
        if !tasks_queue.completed_tasks().is_empty() {
            progress.finish_with_message(format!(
                "{} images were processed and saved with {} errors",
                tasks_queue.completed_tasks().len(),
                tasks_queue.count_failures()
            ));
        }
        Ok(())
    }
}
