use crate::opus::models::Task;

#[derive(Clone, Debug)]
pub enum UndoableAction {
    #[allow(dead_code)]
    TaskCompletion {
        task_id: String,
        previous_state: bool,
    },
    TaskDeletion {
        task: Task,
        position: usize,
    },
    TaskCreation {
        task_id: String,
    },
    TaskEdit {
        task_id: String,
        previous_task: Task,
    },
}
