//! Composing a status, and editing one.
//!
//! Two entry points, one form: `/compose` starts empty, `/compose/{id}` loads the status
//! first. Editing needs the original body, which is why the second entry point exists at
//! all rather than a modal on the card.

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_instance::model::Status;
use nodus_pages::pages::compose::ComposeLoading;

use crate::state::alive::Alive;
use crate::state::bridge::actions;

mod attachments;
mod form;
pub mod inline;
pub(crate) use form::ComposeForm;

/// Writing a new status.
#[component]
pub fn Compose() -> impl IntoView {
    view! { <ComposeForm initial=None/> }
}

/// Editing an existing status.
#[component]
#[expect(
    clippy::needless_pass_by_value,
    reason = "a Leptos component receives its props by value"
)]
pub fn ComposeEdit(id: String) -> impl IntoView {
    let (status, set_status) = signal(None::<Status>);
    let (error, set_error) = signal(None::<String>);

    {
        let id = id.clone();
        Effect::new(move |_| {
            let alive = Alive::new();
            let id = id.clone();
            spawn_local(async move {
                match actions::status(&id).await {
                    Ok(found) => alive.if_alive(|| set_status.set(Some(found))),
                    Err(message) => alive.if_alive(|| set_error.set(Some(message))),
                }
            });
        });
    }

    view! {
        <Show
            when=move || status.get().is_some()
            fallback=move || view! { <ComposeLoading error=error/> }
        >
            {move || status.get().map(|found| view! { <ComposeForm initial=Some(found)/> })}
        </Show>
    }
}
