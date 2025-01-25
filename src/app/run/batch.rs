use super::{command_msg, run_command, RunBatch};
use crate::app::command::{ImageArgs, ImageCommand};
use crate::backend::paths::create_path;
use crate::image::manipulator::{open_image, save_image_format};
use anyhow::{Error, Result};
use crossbeam_channel::{Receiver, Sender};
use image::DynamicImage;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use rayon::iter::{IntoParallelRefIterator, ParallelBridge, ParallelIterator};
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Default, Clone)]
struct ImageTask {
    image: Option<DynamicImage>,
    image_path: PathBuf,
}

impl ImageTask {
    fn new(path: &Path, image: Option<DynamicImage>) -> Self {
        ImageTask {
            image,
            image_path: path.to_path_buf(),
        }
    }
}

#[derive(Clone)]
enum TaskState {
    Decode(String),
    InitProcessing(u64),
    Process(String),
    InitSaving(u64),
    Save(String),
    Complete(u64),
    Failure(String),
}

fn run(command: ImageCommand, args: ImageArgs, verbosity: u32) -> Result<()> {
    let (task_tx, task_rx) = crossbeam_channel::unbounded();
    let (state_tx, state_rx) = crossbeam_channel::unbounded();

    let len = args.images.len() as u64;

    let images = args.images.clone();

    rayon::scope(|s| {
        let decode_sender = state_tx.clone();
        let command = Arc::new(command);
        let args = Arc::new(args);
        let proc_tx = state_tx.clone();
        s.spawn(move |_| {
            decode(images, task_tx, decode_sender);
            let mut tasks = process(command, args.clone(), task_rx, proc_tx);
            save_images(&mut tasks, &state_tx, &args);
        });

        if verbosity != 0 {
            s.spawn(move |_| message(state_rx, len));
        }
    });
    Ok(())
}

fn message(message_reciever: Receiver<TaskState>, length: u64) {
    const PROGRESS_CHARS: &str = "##-";
    let bar = MultiProgress::new();
    let decode_bar = bar.add(
        ProgressBar::new(length).with_style(
            ProgressStyle::with_template(
                "[{pos}/{len}] {msg}\n{bar:40.cyan/blue} [{elapsed_precise}]",
            )
            .unwrap()
            .progress_chars(PROGRESS_CHARS),
        ),
    );

    decode_bar.enable_steady_tick(Duration::from_millis(500));

    let mut process_bar = bar.add(ProgressBar::hidden());
    let mut save_bar = bar.add(ProgressBar::hidden());
    let start_progress = |bar: &mut ProgressBar, length: u64| {
        *bar = ProgressBar::new(length).with_style(
            ProgressStyle::with_template(
                "[{pos}/{len}] {msg}\n{bar:40.cyan/blue} [{elapsed_precise}]",
            )
            .unwrap()
            .progress_chars(PROGRESS_CHARS),
        );
    };

    let start_saving = |bar: &mut ProgressBar, length: u64| {
        *bar = ProgressBar::new(length).with_style(
            ProgressStyle::with_template(
                "[{pos}/{len}] {msg}\n{bar:40.cyan/blue} [{elapsed_precise}]",
            )
            .unwrap()
            .progress_chars(PROGRESS_CHARS),
        );
    };
    save_bar.enable_steady_tick(Duration::from_millis(500));
    while let Ok(state) = message_reciever.recv() {
        match state {
            TaskState::Decode(message) => {
                decode_bar.set_message(message);
                decode_bar.inc(1);
            }
            TaskState::InitProcessing(num) => {
                decode_bar.finish_with_message(format!(
                    "Decoded {num} images with {} errors.",
                    length - num
                ));
                start_progress(&mut process_bar, num);
            }
            TaskState::Process(message) => {
                process_bar.set_message(message);
                process_bar.inc(1);
            }
            TaskState::InitSaving(num) => {
                let length = process_bar.length().unwrap_or(length);
                process_bar.finish_with_message(format!(
                    "Processed {num} images with {} errors",
                    length - num
                ));
                start_saving(&mut save_bar, num);
            }
            TaskState::Save(message) => {
                save_bar.set_message(message);
                save_bar.inc(1);
            }
            TaskState::Complete(num) => {
                let length = save_bar.length().unwrap_or(length);
                save_bar.finish_with_message(format!(
                    "Saved {num} images with {} errors",
                    length - num
                ));
            }
            TaskState::Failure(message) => {
                bar.println(message).unwrap();
            }
        }
    }
}

fn decode(image_paths: Vec<PathBuf>, task_tx: Sender<ImageTask>, message_tx: Sender<TaskState>) {
    let acc = AtomicUsize::new(0);
    image_paths.par_iter().for_each(|image_path| {
        let result = open_image(image_path);

        match result {
            Ok(good_image) => {
                let task = ImageTask::new(image_path, Some(good_image));
                message_tx
                    .send(TaskState::Decode(format!(
                        "{:?}",
                        task.image_path.file_name().as_slice()
                    )))
                    .unwrap_or(());
                acc.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                task_tx.send(task).unwrap_or_else(|_| {
                    println!("Task failed to send!");
                });
            }
            Err(decode_error) => {
                message_tx
                    .send(TaskState::Failure(format!(
                        "Failed to decode: {:?}\nErr:{:?}",
                        image_path, decode_error
                    )))
                    .unwrap_or(());
            }
        }
    });
    message_tx
        .send(TaskState::InitProcessing(acc.into_inner() as u64))
        .unwrap_or(());
}

fn process(
    command: Arc<ImageCommand>,
    args: Arc<ImageArgs>,
    task_rx: Receiver<ImageTask>,
    message_tx: Sender<TaskState>,
) -> Vec<ImageTask> {
    let mut tasks_vec: Vec<ImageTask> = Vec::new();
    while let Ok(task) = task_rx.recv() {
        tasks_vec.push(task);
    }

    let tasks_vec = tasks_vec.par_iter_mut().filter_map(|task| {
        let result = if let Some(image) = task.image.take() {
            run_command(command.deref(), image, args.format.as_deref())
        } else {
            Err(Error::msg("Something happened"))
        };
        match result {
            Ok(image) => {
                let message = command_msg(
                    &command,
                    &format!("{:?}", task.image_path.file_name().as_slice()),
                )
                .unwrap_or(String::from("Error creating message"));
                message_tx.send(TaskState::Process(message)).unwrap_or(());
                task.image = Some(image);
                Some(task.to_owned())
            }
            Err(error) => {
                message_tx
                    .send(TaskState::Failure(format!("Failed operation: {:?}", error)))
                    .unwrap_or(());
                None
            }
        }
    });
    message_tx
        .send(TaskState::InitSaving(
            tasks_vec.opt_len().unwrap_or(0) as u64
        ))
        .unwrap_or(());
    tasks_vec.collect()
}

fn save_images(tasks: &mut Vec<ImageTask>, message_tx: &Sender<TaskState>, args: &ImageArgs) {
    let destination = args.output.clone().unwrap_or(PathBuf::from("."));
    let acc = AtomicUsize::new(0);
    let paths: Vec<PathBuf> = create_paths(
        tasks
            .par_iter()
            .map(|task| task.image_path.to_path_buf())
            .collect(),
        destination,
        args.name_expr.as_deref(),
        args.format.as_deref(),
    )
    .unwrap();

    match save_image_format(&image, &path, args.format.as_deref()) {
        Ok(()) => match message_tx.send(TaskState::Save(format!("Image saved:{:?}", path))) {
            Ok(_) => {
                drop(image);
                Ok(())
            }
            Err(_) => Ok(()),
        },
        Err(e) => match message_tx.send(TaskState::Failure(format!("Error: {:?}", e))) {
            Ok(_) => Ok(()),
            Err(_) => Ok(()),
        },
    }
    message_tx
        .send(TaskState::Complete(acc.into_inner() as u64))
        .unwrap_or(());
}

impl RunBatch for ImageArgs {
    fn run_batch(&self, command: &ImageCommand, verbosity: u32) -> Result<()> {
        run(command.clone(), self.clone(), verbosity)
    }
}
