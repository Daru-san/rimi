#[derive(Debug)]
pub struct AppState {
    pub global_progress: u32,
    tasks: Vec<Task>,
}

#[derive(Debug, Clone)]
struct Task {
    id: u32,
    name: String,
    progress: u32,
    state: TaskState,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            global_progress: 0,
            tasks: Vec::new(),
        }
    }
    pub fn push_task(&mut self, task_name: &str) {
        let task_id = (self.tasks.len() + 1) as u32;
        let task = Task::new(task_id, task_name);
        self.tasks.push(task);
    }
    pub fn task_by_id(&self, task_id: u32) -> Result<Task, String> {
        for task in &self.tasks {
            if task.id == task_id {
                return Ok(task.clone());
            }
        }
        Err("".to_string())
    }
    pub fn start_task(&mut self, task_id: u32) {
        for task in &mut self.tasks {
            if task_id == task.id {
                task.state = TaskState::BUSY;
            }
        }
    }
    pub fn end_task(&mut self, task_id: u32) {
        for task in &mut self.tasks {
            if task_id == task.id {
                task.state = TaskState::DONE;
            }
        }
    }
}

#[derive(Debug, Clone)]
enum TaskState {
    BUSY,
    DONE,
    WAITING,
    NONE,
}

impl Task {
    pub fn new(task_id: u32, task_name: &str) -> Task {
        Task {
            id: task_id,
            name: task_name.to_string(),
            progress: 0,
            state: TaskState::NONE,
        }
    }
    pub fn set_progress() {}
}

fn tasks() {
    let mut app_state = AppState {
        global_progress: 0,
        tasks: Vec::new(),
    };
    app_state.push_task("Read image");
    app_state.start_task(1);
    app_state.push_task("Write file");
    app_state.end_task(1);
}
