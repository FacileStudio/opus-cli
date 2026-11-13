use crate::opus::models::Task;

#[derive(Clone, Debug)]
pub enum UndoableAction {
    Deletion {
        task: Task,
        position: usize,
    },
    Creation {
        task_id: String,
    },
    Edit {
        task_id: String,
        previous_task: Task,
    },
}
