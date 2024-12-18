use std::mem::take;
use std::path::{Path, PathBuf};

use image::DynamicImage;

use super::error::TaskError;

#[derive(Debug, Default, PartialEq, Clone)]
pub enum TaskState {
    #[default]
    Pending,
    Decoded,
    Processed,
    Working,
    Failed(TaskError),
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct ImageTask {
    pub id: u32,
    pub image: DynamicImage,
    pub image_path: PathBuf,
    pub out_path: PathBuf,
    pub state: TaskState,
}

impl ImageTask {
    pub fn new(id: u32, path: &Path) -> Self {
        ImageTask {
            id,
            image: DynamicImage::default(),
            image_path: path.to_path_buf(),
            out_path: path.to_path_buf(),
            state: TaskState::Pending,
        }
    }
}

pub struct TaskQueue {
    tasks: Vec<ImageTask>,
    next_id: u32,
}

impl TaskQueue {
    pub fn new() -> TaskQueue {
        TaskQueue {
            tasks: Vec::new(),
            next_id: 1,
        }
    }

    pub fn new_task(&mut self, path: &Path) -> u32 {
        let task = ImageTask::new(self.next_id, path);
        self.tasks.push(task);
        self.next_id += 1;
        self.next_id - 1
    }

    pub fn task_by_id_mut(&mut self, task_id: u32) -> Option<&mut ImageTask> {
        self.tasks.iter_mut().find(|task| task.id == task_id)
    }

    pub fn working_task(&mut self, task_id: u32) {
        self.tasks.iter_mut().try_for_each(|task| {
            if task.id == task_id {
                task.state = TaskState::Working;
                None
            } else {
                Some(())
            }
        });
    }

    pub fn completed_task(&mut self, task_id: u32) {
        self.tasks.retain(|task| task.id != task_id);
    }

    pub fn decoded_task(&mut self, decoded_image: &mut DynamicImage, task_id: u32) {
        self.tasks.iter_mut().try_for_each(|task| {
            if task.id == task_id {
                task.state = TaskState::Decoded;
                task.image = take(decoded_image);
                None
            } else {
                Some(())
            }
        });
    }

    pub fn fail_task(&mut self, task_id: u32, task_error: String) {
        self.tasks.iter_mut().try_for_each(|task| {
            if task.id == task_id {
                task.state = TaskState::Failed(TaskError::SingleError(task_error.to_string()));
                None
            } else {
                Some(())
            }
        });
    }

    pub fn processed_task(&mut self, processed_image: &mut DynamicImage, task_id: u32) {
        self.tasks.iter_mut().try_for_each(|task| {
            if task.id == task_id {
                task.state = TaskState::Processed;
                task.image = take(processed_image);
                None
            } else {
                Some(())
            }
        });
    }

    pub fn has_failures(&self) -> bool {
        self.tasks
            .iter()
            .any(|task| matches!(task.state, TaskState::Failed(_)))
    }

    pub fn count_failures(&self) -> usize {
        self.failed_tasks().len()
    }

    pub fn failed_tasks(&self) -> Vec<&ImageTask> {
        self.tasks
            .iter()
            .filter(|task| matches!(task.state, TaskState::Failed(_)))
            .collect()
    }

    pub fn processed_tasks(&self) -> Vec<&ImageTask> {
        self.tasks
            .iter()
            .filter(|task| matches!(task.state, TaskState::Processed))
            .collect()
    }

    pub fn decoded_tasks(&self) -> Vec<&ImageTask> {
        self.tasks
            .iter()
            .filter(|task| matches!(task.state, TaskState::Decoded))
            .collect()
    }

    pub fn decoded_task_ids(&self) -> Vec<u32> {
        self.tasks
            .iter()
            .filter(|task| matches!(task.state, TaskState::Decoded))
            .map(|task| task.id)
            .collect()
    }

    pub fn set_task_out_path(&mut self, task_id: u32, path: &mut PathBuf) {
        self.tasks.iter_mut().try_for_each(|task| {
            if task.id == task_id {
                task.out_path = take(path);
                None
            } else {
                Some(())
            }
        });
    }
}
