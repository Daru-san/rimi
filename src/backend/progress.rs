use std::time::Duration;

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

pub trait AppProgress {
    fn init() -> Self;
    fn start_operation(&self, message: &str);
    fn complete_operation_with_message(&self, message: &str);
    fn abort_message(&self, message: &str);
    fn complete(&self);
}

#[derive(Debug)]
pub struct SingleProgress {
    progress_bar: ProgressBar,
    total_progress: u32,
    total_errors: u32,
}

impl AppProgress for SingleProgress {
    fn init() -> SingleProgress {
        let progress_bar = ProgressBar::new(4).with_style(
            ProgressStyle::with_template(
                "[{pos}/{len}] {msg}\n{bar:40.cyan/blue} [{elapsed_precise}]",
            )
            .unwrap()
            .progress_chars("==> "),
        );
        progress_bar.enable_steady_tick(Duration::from_millis(100));
        SingleProgress {
            progress_bar,
            total_progress: 4,
            total_errors: 0,
        }
    }

    fn start_operation(&self, message: &str) {
        self.progress_bar.set_message(message.to_string());
    }

    fn complete_operation_with_message(&self, message: &str) {
        let message = format!(
            "[{}/{}] {}",
            self.progress_bar.position() + 1,
            self.total_progress,
            message
        );
        self.progress_bar.println(message);
        self.progress_bar.inc(1);
    }

    fn abort_message(&self, message: &str) {
        self.progress_bar.abandon_with_message(message.to_string());
    }

    fn complete(&self) {
        self.progress_bar
            .finish_with_message("Image operations completed successfully!");
    }
}

#[derive(Debug)]
pub struct BatchProgress {
    total_progress: u32,
    total_errors: u32,
    current_progress_bar: ProgressBar,
    total_progress_bar: ProgressBar,
    shared_progress_bar: MultiProgress,
}

impl AppProgress for BatchProgress {
    fn init() -> Self {
        let current_progress_bar = ProgressBar::new_spinner();
        current_progress_bar.enable_steady_tick(Duration::from_millis(100));
        let total_progress_bar = ProgressBar::new(4).with_style(
            ProgressStyle::with_template(
                "[{pos}/{len}] {msg}\n{bar:40.cyan/blue} [{elapsed_precise}]",
            )
            .unwrap()
            .progress_chars("==> "),
        );
        total_progress_bar.enable_steady_tick(Duration::from_millis(100));

        let shared_progress_bar = MultiProgress::new();

        let current_progress_bar = shared_progress_bar.add(current_progress_bar);
        let total_progress_bar = shared_progress_bar.add(total_progress_bar);

        Self {
            total_progress: 0,
            total_errors: 0,
            current_progress_bar,
            total_progress_bar,
            shared_progress_bar,
        }
    }
    fn start_operation(&self, message: &str) {
        self.total_progress_bar.set_message(message.to_string());
    }
    fn complete_operation_with_message(&self, message: &str) {
        self.current_progress_bar.println(format!(
            "[{}/{}] Task complete: {}",
            self.total_progress_bar.position(),
            self.total_progress,
            message
        ));
    }
    fn abort_message(&self, message: &str) {
        self.total_progress_bar
            .abandon_with_message(message.to_string());
    }
    fn complete(&self) {
        self.total_progress_bar.finish_and_clear();
    }
}

impl BatchProgress {
    pub fn start_sub_operation(&self, message: &str) {
        self.current_progress_bar.set_message(message.to_string());
    }
    pub fn next_sub(&self) {
        self.current_progress_bar.inc(1);
    }
    pub fn complete_sub_operation(&self, message: &str) {
        self.current_progress_bar.println(message.to_string());
    }
    pub fn error_sub_operation(&self, message: &str) {
        self.current_progress_bar
            .println(format!("Process exited with error: {}", message));
    }
}
