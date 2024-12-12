use std::time::Duration;

use indicatif::{ProgressBar, ProgressStyle};

#[derive(Debug)]
pub struct GlobalProgress {
    progress_bar: ProgressBar,
    total_progress: u32,
}

impl GlobalProgress {
    pub fn init() -> GlobalProgress {
        let progress_bar = ProgressBar::new(4).with_style(
            ProgressStyle::with_template(
                "[{pos}/{len}] {msg}\n{bar:40.cyan/blue} [{elapsed_precise}]",
            )
            .unwrap()
            .progress_chars("==> "),
        );
        progress_bar.enable_steady_tick(Duration::from_millis(100));
        GlobalProgress {
            progress_bar,
            total_progress: 4,
        }
    }

    pub fn set_current_operation(&self, message: &str) {
        self.progress_bar.set_message(message.to_string());
    }

    pub fn complete_operation_with_message(&self, message: &str) {
        let message = format!(
            "[{}/{}] {}",
            self.progress_bar.position() + 1,
            self.total_progress,
            message
        );
        self.progress_bar.println(message);
        self.progress_bar.inc(1);
    }

    pub fn abort_message(&self, message: &str) {
        self.progress_bar.abandon_with_message(message.to_string());
    }

    pub fn complete(&self) {
        self.progress_bar
            .finish_with_message("Image operations completed successfully!");
    }
}
