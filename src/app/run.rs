use super::command::ImageCommand;

mod batch;
mod single;

pub trait RunSingle {
    fn run_single(&self, command: &ImageCommand, verbosity: u32) -> anyhow::Result<()>;
}

pub trait RunBatch {
    fn run_batch(
        &self,
        command: &ImageCommand,
        verbosity: u32,
        task_count: usize,
    ) -> anyhow::Result<()>;
}
