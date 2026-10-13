/// Modal subsystem for TUI. All modal openers must use the centralized viewport guard in `utils.rs`.
/// This ensures modals do not open or crash in undersized terminals, and instead show a user message.
///
/// Maintainability: Modal draw functions should not contain viewport size checks. All guards are centralized.
/// See `try_show_modal` in `utils.rs` for details and constants.
pub mod utils;

mod quick_add;
mod edit;
mod form_edit;
mod comments;

pub use quick_add::handle_quick_add_modal;
pub use edit::handle_edit_modal;
pub use form_edit::handle_form_edit_modal;
pub use comments::{CommentsModal, CommentsModalAction};
