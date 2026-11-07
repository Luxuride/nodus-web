//! The starting values of the composer, read from the status being edited.

use nodus_instance::model::{Status, Visibility};

/// Everything the form needs to know before the first keystroke.
pub struct Initial {
    /// Whether this is an edit, and of what.
    pub editing: Option<String>,
    /// The body, in plain text.
    pub text: String,
    /// The visibility it was posted with.
    pub visibility: Visibility,
    /// The content warning, if any.
    pub spoiler: String,
    /// Whether the warning field starts open.
    pub warning_open: bool,
    /// Whether the media was marked sensitive.
    pub sensitive: bool,
}

impl Initial {
    /// Reads a status, or returns the blank draft.
    pub fn of(status: Option<&Status>) -> Self {
        let Some(status) = status else {
            return Self {
                editing: None,
                text: String::new(),
                visibility: Visibility::Public,
                spoiler: String::new(),
                warning_open: false,
                sensitive: false,
            };
        };
        Self {
            editing: Some(status.id.clone()),
            text: crate::routes::feed::posts::plain_text(&status.content),
            visibility: status.visibility,
            spoiler: status.spoiler_text.clone(),
            warning_open: !status.spoiler_text.is_empty(),
            sensitive: status.sensitive,
        }
    }
}
