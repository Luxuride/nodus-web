//! A poll, as the composer builds one.
//!
//! Kept apart from the composer body because it is a small form of its own: two to four
//! options, how long they stay open, and whether a voter may pick more than one. The
//! composer owns the state so the poll survives a re-render of this component, and this
//! module only renders it and reports edits upward — a component that owned the options
//! would lose them the moment the panel closed.

use nodus_client_core::model::PollDraft;
use wasm_bindgen::JsCast as _;

/// The poll's answer as the wire wants it, or `None` when it is not one yet.
///
/// Fewer than two non-empty options is not a poll: the instance rejects it, and asking a
/// question with one answer is a statement. Returning `None` here keeps that rule in one
/// place instead of inside the submit handler.
#[must_use]
pub(super) fn as_draft(
    open: bool,
    options: &[String],
    expires_in: u32,
    multiple: bool,
    hide_totals: bool,
) -> Option<PollDraft> {
    if !open {
        return None;
    }
    let filled: Vec<String> = options
        .iter()
        .map(|option| option.trim().to_owned())
        .filter(|option| !option.is_empty())
        .collect();
    if filled.len() < 2 {
        return None;
    }
    Some(
        PollDraft::new(filled, expires_in)
            .multiple(multiple)
            .hide_totals(hide_totals),
    )
}

pub(super) use nodus_pages::pages::compose::inline::poll::PollEditor;

/// Puts the caret in the option that was just added.
///
/// `id` rather than a `NodeRef`: the rows are keyed by index, so one ref would have to be
/// shared by every input and would end up pointing at whichever rendered last.
pub(super) fn focus_option(index: usize) {
    let Some(document) = web_sys::window().and_then(|window| window.document()) else {
        return;
    };
    let id = format!("poll-option-{index}");
    if let Some(field) = document
        .get_element_by_id(&id)
        .and_then(|element| element.dyn_into::<web_sys::HtmlElement>().ok())
    {
        let _ = field.focus();
    }
}
