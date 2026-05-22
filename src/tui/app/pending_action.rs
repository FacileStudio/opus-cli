#[derive(Clone, Debug)]
pub enum PendingAction {
    DeleteTask { task_id: String },
    QuitApp,
}
