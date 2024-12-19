use std::time::{Duration, Instant};

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

pub trait AppProgressBar {
    fn init(verbosity: u32, size: usize) -> Self;
    fn start_task(&self, message: &str);
    fn abort(&self, message: &str);
    fn spawn_new(&self, _: usize, _: &str) {}
    #[allow(dead_code)]
    fn next(&self) {}
    fn message(&self, message: &str);
    fn send_trace(&self, message: &str);
    fn suspend<F: FnOnce() -> R, R>(&self, f: F) -> R;
    fn exit(&self);
}

const PROGRESS_CHARS: &str = "##-";

#[derive(Debug)]
pub struct SingleProgressBar {
    progress_bar: ProgressBar,
    start_time: Instant,
}

impl AppProgressBar for SingleProgressBar {
    fn init(verbosity: u32, size: usize) -> SingleProgressBar {
        let progress_bar = if verbosity == 0 {
            ProgressBar::hidden()
        } else {
            let bar = ProgressBar::new(size as u64).with_style(
                ProgressStyle::with_template(
                    "[{pos}/{len}] {msg}\n[{bar:40.cyan/blue}] [{elapsed_precise}]",
                )
                .unwrap()
                .progress_chars(PROGRESS_CHARS),
            );
            bar.enable_steady_tick(Duration::from_millis(100));
            bar
        };

        let start_time = Instant::now();

        SingleProgressBar {
            progress_bar,
            start_time,
        }
    }

    fn start_task(&self, message: &str) {
        self.progress_bar.set_message(message.to_string());
        self.progress_bar.inc(1);
    }

    fn next(&self) {
        self.progress_bar.inc(1);
    }

    fn abort(&self, message: &str) {
        self.progress_bar.abandon_with_message(message.to_string());
    }

    fn message(&self, message: &str) {
        self.progress_bar.set_message(message.to_string());
    }

    fn send_trace(&self, message: &str) {
        self.progress_bar.println(message);
    }

    fn exit(&self) {
        let now = Instant::now();
        let total_duration = now.duration_since(self.start_time);

        self.progress_bar.finish_with_message(format!(
            "image operations completed in {}s!",
            total_duration.as_secs()
        ));
    }

    fn suspend<F: FnOnce() -> R, R>(&self, f: F) -> R {
        self.progress_bar.suspend(f)
    }
}

#[derive(Debug)]
pub struct BatchProgressBar {
    total_errors: u32,

    subtask_errors: u32,

    start_time: Instant,

    subtask_progress: ProgressBar,
    task_progress: ProgressBar,

    #[allow(dead_code)]
    multi_progress: MultiProgress,
}

impl AppProgressBar for BatchProgressBar {
    fn init(verbosity: u32, size: usize) -> Self {
        let multi_progress = MultiProgress::new();

        let (task_progress, subtask_progress);

        if verbosity == 0 {
            task_progress = ProgressBar::hidden();
            subtask_progress = ProgressBar::hidden();
        } else {
            task_progress = multi_progress.add(ProgressBar::new_spinner().with_style(
                ProgressStyle::with_template("[{pos}/{len}] {spinner} {msg}").unwrap(),
            ));
            task_progress.set_length(size as u64);

            subtask_progress = multi_progress.add(
                ProgressBar::new(0).with_style(
                    ProgressStyle::with_template(
                        "[{pos}/{len}] {msg}\n{bar:40.cyan/blue} [{elapsed_precise}]",
                    )
                    .unwrap()
                    .progress_chars(PROGRESS_CHARS),
                ),
            );

            subtask_progress.enable_steady_tick(Duration::from_millis(100));

            task_progress.enable_steady_tick(Duration::from_millis(100));
        }

        Self {
            total_errors: 0,

            subtask_errors: 0,

            subtask_progress,
            task_progress,
            multi_progress,

            start_time: Instant::now(),
        }
    }

    fn spawn_new(&self, size: usize, message: &str) {
        self.subtask_progress.finish();
        self.subtask_progress.set_length(size as u64);
        self.subtask_progress.set_position(0);

        self.task_progress.set_message(message.to_string());
        self.task_progress.inc(1);
    }

    fn start_task(&self, message: &str) {
        self.subtask_progress.set_message(message.to_string());
        self.subtask_progress.inc(1);
    }

    fn next(&self) {
        self.task_progress.inc(1);
    }

    fn message(&self, message: &str) {
        self.task_progress.set_message(message.to_string());
    }

    fn abort(&self, message: &str) {
        self.subtask_progress.abandon();
        self.task_progress.abandon_with_message(message.to_string());
    }

    fn suspend<F: FnOnce() -> R, R>(&self, f: F) -> R {
        self.multi_progress.suspend(f)
    }

    fn send_trace(&self, message: &str) {
        self.multi_progress.println(message).unwrap();
    }

    fn exit(&self) {
        let now = Instant::now();
        let total_duration = now.duration_since(self.start_time);

        if let Some(completed_sub_tasks) = self.subtask_progress.length() {
            if self.subtask_errors == 0 {
                self.subtask_progress.finish_with_message(format!(
                    "{} images were saved successfully.",
                    completed_sub_tasks
                ));
            } else {
                self.subtask_progress.finish_with_message(format!(
                    "{} images were saved with {} errors.",
                    completed_sub_tasks - (self.subtask_errors as u64),
                    self.subtask_errors
                ));
            }
        }

        if let Some(completed_tasks) = self.task_progress.length() {
            self.task_progress.finish_with_message(format!(
                "{} tasks completed with {} errors in {}s",
                completed_tasks,
                self.total_errors,
                total_duration.as_secs()
            ));
        }
    }
}
