use std::time::{Duration, Instant};

use indicatif::{ProgressBar, ProgressStyle};

pub trait AppProgressBar {
    fn init(verbosity: u32, size: usize) -> Self;
    fn start_task(&self, message: &str);
    fn abort(&self, message: &str);
    fn message(&self, message: &str);
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

    fn abort(&self, message: &str) {
        self.progress_bar.abandon_with_message(message.to_string());
    }

    fn message(&self, message: &str) {
        self.progress_bar.set_message(message.to_string());
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
