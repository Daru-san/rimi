use std::time::{Duration, Instant};

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

pub trait AppProgress {
    fn init(verbosity: u32) -> Self;
    fn start_task(&self, message: &str);
    fn finish_task(&self, message: &str);
    fn abort_task(&self, message: &str);
    fn exit(&self);
}

const PROGRESS_CHARS: &str = "##-";

#[derive(Debug)]
pub struct SingleProgress {
    progress_bar: ProgressBar,
    start_time: Instant,
    total_progress: u32,
    verbosity: u32,
}

impl AppProgress for SingleProgress {
    fn init(verbosity: u32) -> SingleProgress {
        let progress_bar = ProgressBar::new(4).with_style(
            ProgressStyle::with_template(
                "[{pos}/{len}] {msg}\n[{bar:40.cyan/blue}] [{elapsed_precise}]",
            )
            .unwrap()
            .progress_chars(PROGRESS_CHARS),
        );
        progress_bar.enable_steady_tick(Duration::from_millis(100));

        if verbosity == 0 {
            progress_bar.finish_and_clear();
        }

        let start_time = Instant::now();

        SingleProgress {
            progress_bar,
            total_progress: 4,
            start_time,
            verbosity,
        }
    }

    fn start_task(&self, message: &str) {
        self.progress_bar.set_message(message.to_string());
    }

    fn finish_task(&self, message: &str) {
        let message = format!(
            "[{}/{}] {}",
            self.progress_bar.position() + 1,
            self.total_progress,
            message
        );
        if self.verbosity == 2 {
            self.progress_bar.println(message);
        } else {
            self.progress_bar.set_message(message);
        }
        self.progress_bar.inc(1);
    }

    fn abort_task(&self, message: &str) {
        self.progress_bar.abandon_with_message(message.to_string());
    }

    fn exit(&self) {
        let now = Instant::now();
        let total_duration = now.duration_since(self.start_time);

        self.progress_bar.finish_with_message(format!(
            "image operations completed in {}!",
            total_duration.as_secs()
        ));
    }
}

#[derive(Debug)]
pub struct BatchProgress {
    total_errors: u32,

    start_time: Instant,

    verbosity: u32,

    subtask_progress: ProgressBar,
    task_progress: ProgressBar,

    #[allow(dead_code)]
    multi_progress: MultiProgress,
}

impl AppProgress for BatchProgress {
    fn init(verbosity: u32) -> Self {
        let multi_progress = MultiProgress::new();

        let subtask_progress = multi_progress
            .add(ProgressBar::new_spinner().with_style(
                ProgressStyle::with_template("[{pos}/{len}] {spinner} {msg}").unwrap(),
            ));

        let task_progress = multi_progress.add(
            ProgressBar::new(4).with_style(
                ProgressStyle::with_template(
                    "[{pos}/{len}] {msg}\n{bar:40.cyan/blue} [{elapsed_precise}]",
                )
                .unwrap()
                .progress_chars(PROGRESS_CHARS),
            ),
        );

        subtask_progress.enable_steady_tick(Duration::from_millis(100));

        task_progress.enable_steady_tick(Duration::from_millis(100));

        if verbosity == 0 {
            subtask_progress.finish_and_clear();
            task_progress.finish_and_clear();
            multi_progress.remove(&subtask_progress);
            multi_progress.remove(&task_progress);
        }

        Self {
            total_errors: 0,

            verbosity,

            subtask_progress,
            task_progress,
            multi_progress,

            start_time: Instant::now(),
        }
    }
    fn start_task(&self, message: &str) {
        self.task_progress.set_message(message.to_string());
    }
    fn finish_task(&self, message: &str) {
        if let Some(total_progress) = self.task_progress.length() {
            let message = format!(
                "[{}/{}] Task complete: {}",
                self.task_progress.position() + 1,
                total_progress,
                message
            );

            self.subtask_progress.println(message);
        }
        self.task_progress.inc(1);
    }
    fn abort_task(&self, message: &str) {
        self.subtask_progress.abandon();
        self.task_progress.abandon_with_message(message.to_string());
    }
    fn exit(&self) {
        let now = Instant::now();
        let total_duration = now.duration_since(self.start_time);

        self.subtask_progress.finish();

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

impl BatchProgress {
    pub fn start_sub_task(&self, message: &str) {
        self.subtask_progress.set_message(message.to_string());
    }
    pub fn finish_sub_task(&self, message: &str) {
        if let Some(total_progress) = self.subtask_progress.length() {
            let message = format!(
                "[{}/{}] {}",
                self.subtask_progress.position() + 1,
                total_progress,
                message
            );
            if self.verbosity == 2 {
                self.subtask_progress.println(message);
            } else {
                self.subtask_progress.set_message(message);
            }
        }
        self.subtask_progress.inc(1);
    }
    pub fn error_sub_task(&mut self, message: &str) {
        if let Some(total_progress) = self.subtask_progress.length() {
            self.subtask_progress.println(format!(
                "[{}/{}] {}",
                self.subtask_progress.position(),
                total_progress,
                message
            ));
        }
        self.subtask_progress.inc(1);
        self.total_errors += 1;
    }
    pub fn task_count(&mut self, count: usize) {
        self.task_progress.set_position(0);
        self.task_progress.set_length(count as u64);
    }
    pub fn sub_task_count(&self, count: usize) {
        self.subtask_progress.set_position(0);
        self.subtask_progress.set_length(count as u64);
    }
}
