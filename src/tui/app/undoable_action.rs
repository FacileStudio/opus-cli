use crate::opus::models::Task;

#[derive(Clone, Debug)]
pub enum UndoableAction {
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
