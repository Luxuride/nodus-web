//! Carrying a draft from a short composer into the full one.

use std::cell::RefCell;

use nodus_instance::model::MediaAttachment;

thread_local! {
    static DRAFT: RefCell<Option<Handoff>> = const { RefCell::new(None) };
}

pub(crate) struct Handoff {
    pub(crate) text: String,
    pub(crate) reply_to: Option<String>,
    pub(crate) media: Vec<MediaAttachment>,
}

pub(crate) fn put(text: String, reply_to: Option<String>, media: Vec<MediaAttachment>) {
    DRAFT.with(|draft| {
        *draft.borrow_mut() = Some(Handoff {
            text,
            reply_to,
            media,
        })
    });
}

pub(crate) fn take() -> Option<Handoff> {
    DRAFT.with(|draft| draft.borrow_mut().take())
}
