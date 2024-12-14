use std::time::{Duration, Instant};

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

pub trait AppProgress {
    fn init(verbosity: u32) -> Self;
    fn start_task(&self, message: &str);
    fn finish_task(&self, message: &str);
    fn abort_task(&self, message: &str);
    fn exit(&self);
}

const PROGRESS_CHARS: &str = "=>-";

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
        let current_progress_bar = ProgressBar::new_spinner()
            .with_style(ProgressStyle::with_template("[{pos}/{len}] {spinner} {msg}").unwrap());

        current_progress_bar.enable_steady_tick(Duration::from_millis(100));

        let total_progress_bar = ProgressBar::new(4).with_style(
            ProgressStyle::with_template(
                "[{pos}/{len}] {msg}\n{bar:40.cyan/blue} [{elapsed_precise}]",
            )
            .unwrap()
            .progress_chars(PROGRESS_CHARS),
        );

        total_progress_bar.enable_steady_tick(Duration::from_millis(100));

        let shared_progress_bar = MultiProgress::new();

        let current_progress_bar = shared_progress_bar.add(current_progress_bar);

        let total_progress_bar = shared_progress_bar.add(total_progress_bar);

        if verbosity == 0 {
            current_progress_bar.finish_and_clear();
            total_progress_bar.finish_and_clear();
            shared_progress_bar.remove(&current_progress_bar);
            shared_progress_bar.remove(&total_progress_bar);
        }

        Self {
            total_errors: 0,
            completed_tasks: 0,

            verbosity,

            current_progress_bar,
            total_progress_bar,
            shared_progress_bar,

            start_time: Instant::now(),
        }
    }
    fn start_task(&self, message: &str) {
        self.total_progress_bar.set_message(message.to_string());
    }
    fn finish_task(&self, message: &str) {
        if let Some(total_progress) = self.total_progress_bar.length() {
            let message = format!(
                "[{}/{}] Task complete: {}",
                self.total_progress_bar.position() + 1,
                total_progress,
                message
            );

            self.current_progress_bar.println(message);
        }
        self.total_progress_bar.inc(1);
    }
    fn abort_task(&self, message: &str) {
        self.total_progress_bar
            .abandon_with_message(message.to_string());
    }
    fn exit(&self) {
        let now = Instant::now();
        let total_duration = now.duration_since(self.start_time);

        self.current_progress_bar.finish_with_message(format!(
            "{} tasks completed with {} errors in {}s",
            self.completed_tasks,
            self.total_errors,
            total_duration.as_secs()
        ));
        self.total_progress_bar.finish_and_clear();
    }
}

impl BatchProgress {
    pub fn start_sub_task(&self, message: &str) {
        self.current_progress_bar.set_message(message.to_string());
    }
    pub fn finish_sub_task(&self, message: &str) {
        if let Some(total_progress) = self.current_progress_bar.length() {
            let message = format!(
                "[{}/{}] {}",
                self.current_progress_bar.position() + 1,
                total_progress,
                message
            );
            if self.verbosity == 2 {
                self.current_progress_bar.println(message);
            } else {
                self.current_progress_bar.set_message(message);
            }
        }
        self.current_progress_bar.inc(1);
    }
    pub fn error_sub_task(&mut self, message: &str) {
        if let Some(total_progress) = self.current_progress_bar.length() {
            self.current_progress_bar.println(format!(
                "[{}/{}] {}",
                self.current_progress_bar.position(),
                total_progress,
                message
            ));
        }
        self.current_progress_bar.inc(1);
        self.total_errors += 1;
    }
    pub fn task_count(&mut self, count: usize) {
        self.total_progress_bar.set_position(0);
        self.total_progress_bar.set_length(count as u64);
    }
    pub fn sub_task_count(&self, count: usize) {
        self.current_progress_bar.set_position(0);
        self.current_progress_bar.set_length(count as u64);
    }
}
