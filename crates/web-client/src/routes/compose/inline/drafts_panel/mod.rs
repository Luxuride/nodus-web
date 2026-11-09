//! The `:draft` pickup list's shell.
//!
//! Typing `:draft` is a request for what was left unsent, in the same place the emoji
//! suggestions appear — one popup vocabulary for everything the composer can insert. A
//! choice fills the composer with that draft's text, visibility and language, and removes
//! it from the list, because a draft that stays put after being resumed is a draft that
//! gets posted twice.
//!
//! The rows are the view's. What stays is the store behind them: `localStorage` is this
//! crate's, and so is the clock the age column is read from, so the panel is handed rows
//! that are already display-ready.

use leptos::prelude::*;
use nodus_pages::pages::compose::inline::drafts_panel::{Draft, DraftsPanel as Panel};

use crate::ui::drafts::{self, Saved};

/// The open `:draft` popup.
#[component]
pub(super) fn DraftsPanel(
    /// True while the writer's text ends in `:draft`.
    open: Signal<bool>,
    /// Called with the index of the chosen draft.
    insert: Callback<usize>,
) -> impl IntoView {
    let (drafts, set_drafts) = signal(Vec::<Saved>::new());

    // Reload when the popup opens rather than keeping a second copy of `localStorage` in
    // memory: the store changes in another tab as easily as in this one.
    Effect::new(move |_| {
        if open.get() {
            set_drafts.set(drafts::all());
        }
    });

    // `to_iso_string` hands back a `JsString`, and `relative_time` wants the `String` this
    // crate parses dates from — hence a `String` per row rather than a live read in the row.
    let rows = Signal::derive(move || {
        drafts
            .get()
            .into_iter()
            .map(|draft| {
                let when = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(draft.at));
                Draft {
                    label: drafts::summary(&draft.text),
                    visibility: draft.visibility,
                    language: if draft.language.is_empty() {
                        "instance default".to_owned()
                    } else {
                        draft.language
                    },
                    age: crate::routes::format::relative_time(&String::from(when.to_iso_string())),
                }
            })
            .collect()
    });

    view! { <Panel open=open drafts=rows insert=insert/> }
}
