use std::mem::take;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use crate::app::command::{ImageArgs, ImageCommand};
use crate::backend::error::TaskError;
use crate::backend::paths::{create_paths, paths_exist, prompt_overwrite};
use crate::backend::progress::{AppProgressBar, BatchProgressBar};
use crate::image::manipulator::{open_image, save_image_format};

use super::{command_msg, run_command, RunBatch};
use anyhow::Result;
use image::DynamicImage;
use rayon::iter::{
    IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelDrainRange, ParallelIterator,
};

const TASK_COUNT: usize = 4;

#[derive(Debug, Default, PartialEq, Clone)]
struct ImageTask {
    image: DynamicImage,
    image_path: PathBuf,
    out_path: PathBuf,
}

impl ImageTask {
    fn new(path: &Path, image: DynamicImage) -> Self {
        ImageTask {
            image,
            image_path: path.to_path_buf(),
            out_path: path.to_path_buf(),
        }
    }
}

struct BatchRunner {
    tasks_queue: Mutex<Vec<Option<ImageTask>>>,
    progress_bar: Mutex<BatchProgressBar>,
}

impl BatchRunner {
    fn init(verbosity: u32) -> Self {
        Self {
            tasks_queue: Mutex::new(Vec::with_capacity(0)),
            progress_bar: Mutex::new(BatchProgressBar::init(verbosity, TASK_COUNT)),
        }
    }

    fn run(&mut self, command: &ImageCommand, args: &ImageArgs) -> Result<()> {
        self.decode_images(args)?;

        self.outpaths(args)?;

        self.process_images(command, args.format.as_deref())?;

        self.save_images(args)?;

        self.progress_bar.lock().unwrap().exit();

        Ok(())
    }

    fn decode_images(&mut self, args: &ImageArgs) -> Result<()> {
        let tasks_queue = &self.tasks_queue;
        let progress_bar = &self.progress_bar;

        let images = args.images.len() - 1;
        {
            let mut tasks_queue = tasks_queue.lock().unwrap();

            tasks_queue.reserve_exact(images);

            let progress_bar = progress_bar.lock().unwrap();

            progress_bar.spawn_new(images, &format!("Decoding {} images", images));
        }

        args.images.par_iter().for_each(|image_path| {
            if let Ok(progress_bar) = progress_bar.try_lock() {
                progress_bar.start_task(&format!(
                    "Decoding image: {:?}",
                    image_path.file_name().as_slice()
                ));
            }

            let result = open_image(image_path);

            match result {
                Ok(mut good_image) => {
                    let task = ImageTask::new(image_path.as_path(), take(&mut good_image));
                    let mut tasks_queue = tasks_queue.lock().unwrap();
                    tasks_queue.push(Some(task));
                }
                Err(decode_error) => {
                    if let Ok(progress) = progress_bar.try_lock() {
                        progress.send_trace(&format!("Error: {}", decode_error));
                    }
                }
            }
        });

        tasks_queue.lock().unwrap().shrink_to_fit();

        Ok(())
    }

    fn outpaths(&mut self, args: &ImageArgs) -> Result<()> {
        let progress_bar = self.progress_bar.lock().unwrap();
        let mut tasks_queue = self.tasks_queue.lock().unwrap();

        progress_bar.spawn_new(1, "Creating output paths");

        let destination_path = match &args.output {
            Some(path) => path.to_path_buf(),
            None => PathBuf::from("."),
        };

        let input_paths: Vec<PathBuf> = tasks_queue
            .par_iter()
            .flatten_iter()
            .map(|task| task.image_path.to_path_buf())
            .collect();

        let output_paths = match create_paths(
            input_paths,
            destination_path,
            args.name_expr.as_deref(),
            args.format.as_deref(),
        ) {
            Ok(out_paths) => out_paths,
            Err(path_create_error) => {
                progress_bar.abort("Error creating out paths");
                return Err(TaskError::SingleError(path_create_error).into());
            }
        };

        match paths_exist(&output_paths) {
            Ok(paths) => {
                if !paths.is_empty() && !args.overwrite {
                    let prompt = || -> Result<()> {
                        match prompt_overwrite(paths) {
                            Ok(()) => Ok(()),
                            Err(error) => Err(TaskError::SingleError(error).into()),
                        }
                    };
                    progress_bar.suspend(|| -> Result<()> { prompt() })?;
                }
            }
            Err(e) => return Err(TaskError::SingleError(e).into()),
        }

        for (index, task) in tasks_queue.iter_mut().flatten().enumerate() {
            task.out_path = output_paths[index].to_path_buf();
        }

        progress_bar.message("Paths created successfully.");

        Ok(())
    }

    fn process_images(&mut self, command: &ImageCommand, format: Option<&str>) -> Result<()> {
        let tasks_queue = &self.tasks_queue;
        let progress = &self.progress_bar;

        let mut tasks_queue = tasks_queue.lock().unwrap();
        {
            let progress = progress.lock().unwrap();
            progress.spawn_new(
                tasks_queue.len(),
                &format!("Processing {} images", tasks_queue.len()),
            );
        }

        tasks_queue.par_iter_mut().for_each(|task| {
            let failed = false;
            if let Some(task) = task {
                if let Ok(progress_bar) = progress.try_lock() {
                    if let Some(path) = task.image_path.file_name() {
                        progress_bar.start_task(
                            &command_msg(
                                command,
                                path.to_str().unwrap_or(
                                    task.image_path.as_os_str().to_string_lossy().as_ref(),
                                ),
                            )
                            .unwrap_or(String::from("Error formatting this message.")),
                        );
                    }
                }

                let result = run_command(command, &mut task.image, format);

                match result {
                    Ok(new_image) => task.image = new_image,
                    Err(error) => {
                        if let Ok(progress) = progress.try_lock() {
                            progress.send_trace(&format!("Error: {}", error));
                        }
                    }
                };
            }
            if failed {
                *task = None;
            }
        });
        tasks_queue.retain(|task| task.is_some());
        tasks_queue.shrink_to_fit();
        Ok(())
    }
    fn save_images(&mut self, args: &ImageArgs) -> Result<()> {
        let tasks_queue = &self.tasks_queue;
        let progress = &self.progress_bar;

        let mut tasks_queue = tasks_queue.lock().unwrap();
        {
            let progress = progress.lock().unwrap();
            progress.spawn_new(
                tasks_queue.len(),
                &format!("Processing {} images", tasks_queue.len()),
            );
        }

        tasks_queue.par_iter_mut().for_each(|task| {
            if let Some(task) = task {
                if let Ok(progress_bar) = progress.try_lock() {
                    progress_bar.start_task(&format!(
                        "Saving image: {:?}",
                        task.out_path.file_name().as_slice()
                    ));
                }

                let result = save_image_format(&task.image, &task.out_path, args.format.as_deref());

                match result {
                    Ok(()) => (),
                    Err(error) => {
                        if let Ok(progress) = progress.try_lock() {
                            progress.send_trace(&format!("Error: {}", error));
                        }
                    }
                };
            }
            *task = None;
        });
        tasks_queue.par_drain(..);
        Ok(())
    }
}

impl RunBatch for ImageArgs {
    fn run_batch(&self, command: &ImageCommand, verbosity: u32) -> Result<()> {
        BatchRunner::init(verbosity).run(command, self)
    }
}
