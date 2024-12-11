use std::error::Error;
use std::fmt::Display;
use std::path::PathBuf;

use anyhow::Result;
use image::DynamicImage;

#[derive(Debug)]
pub struct BatchErrors(pub Vec<TaskError>);

impl Error for BatchErrors {}

impl Display for BatchErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Errors occured while parsing images: {}", self.0.len())?;
        for err in &self.0 {
            writeln!(f, "{}", err.0)?
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub struct TaskError(pub String);

impl Error for TaskError {}

impl Display for TaskError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Image processing failed due to error: {}", self.0)
    }
}

    }
}

#[derive(Debug, PartialEq)]
pub enum TaskState {
    Pending,
    Decoded,
    Processed,
    Failed(TaskError),
}

#[derive(Debug, PartialEq)]
pub struct ImageTask {
    pub id: u32,
    pub image: DynamicImage,
    pub image_path: PathBuf,
    pub out_path: PathBuf,
    pub state: TaskState,
}

impl ImageTask {
    pub fn new(id: u32, path: &PathBuf) -> Self {
        ImageTask {
            id,
            image: DynamicImage::default(),
            image_path: path.to_path_buf(),
            out_path: path.to_path_buf(),
            state: TaskState::Pending,
        }
    }
}

#[derive(Debug)]
pub struct TaskQueue {
    pub tasks: Vec<ImageTask>,
    pub next_id: u32,
}

impl TaskQueue {
    pub fn new() -> TaskQueue {
        TaskQueue {
            tasks: Vec::new(),
            next_id: 1,
        }
    }

    pub fn new_task(&mut self, path: &PathBuf) -> u32 {
        let task = ImageTask::new(self.next_id, path);
        self.tasks.push(task);
        self.next_id += 1;
        self.next_id - 1
    }

    pub fn task_by_id_mut(&mut self, task_id: u32) -> Option<&mut ImageTask> {
        self.tasks.iter_mut().find(|task| task.id == task_id)
    }

    pub fn task_by_id(&self, task_id: u32) -> Option<&ImageTask> {
        self.tasks.iter().find(|task| task.id == task_id)
    }

    pub fn set_decoded(&mut self, decoded_image: &DynamicImage, task_id: u32) {
        for task in self.tasks.iter_mut() {
            if task.id == task_id {
                task.state = TaskState::Decoded;
                task.image = decoded_image.clone();
            }
        }
    }

    pub fn set_failed(&mut self, task_id: u32, task_error: String) {
        for task in self.tasks.iter_mut() {
            if task.id == task_id {
                task.state = TaskState::Failed(TaskError(task_error.to_string()));
            }
        }
    }

    pub fn set_processed(&mut self, processed_image: &DynamicImage, task_id: u32) {
        for task in self.tasks.iter_mut() {
            if task.id == task_id {
                task.state = TaskState::Processed;
                task.image = processed_image.clone();
            }
        }
    }

    pub fn has_failures(&self) -> bool {
        for task in &self.tasks {
            match task.state {
                TaskState::Failed(_) => return true,
                _ => (),
            }
        }
        false
    }

    pub fn count_failures(&self) -> usize {
        if self.tasks.len() != 0 {
            self.tasks.len() - 1
        } else {
            0
        }
    }

    pub fn failed_tasks(&self) -> Vec<&ImageTask> {
        let mut tasks = Vec::new();
        for task in &self.tasks {
            match task.state {
                TaskState::Failed(_) => tasks.push(task),
                _ => (),
            }
        }
        tasks
    }

    pub fn decoded_tasks(&self) -> Vec<&ImageTask> {
        let mut tasks = Vec::new();
        for task in &self.tasks {
            match task.state {
                TaskState::Decoded => tasks.push(task),
                _ => (),
            }
        }
        tasks
    }

    pub fn decoded_ids(&self) -> Vec<u32> {
        let mut ids = Vec::new();
        for task in &self.tasks {
            match task.state {
                TaskState::Decoded => ids.push(task.id),
                _ => (),
            }
        }
        ids
    }

    pub fn set_out_path(&mut self, task_id: u32, path: &PathBuf) {
        for task in self.tasks.iter_mut() {
            if task.id == task_id {
                task.out_path = path.to_path_buf();
            }
        }
    }
}
