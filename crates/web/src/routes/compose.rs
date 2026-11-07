//! Composing a status, and editing one.
//!
//! Two entry points, one form: `/compose` starts empty, `/compose/{id}` loads the status
//! first. Editing needs the original body, which is why the second entry point exists at
//! all rather than a modal on the card.

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_instance::model::Status;

use crate::state::bridge::actions;

mod attachments;
mod form;

use form::ComposeForm;

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
            let id = id.clone();
            spawn_local(async move {
                match actions::status(&id).await {
                    Ok(found) => set_status.set(Some(found)),
                    Err(message) => set_error.set(Some(message)),
                }
            });
        });
    }

    view! {
        <Show
            when=move || status.get().is_some()
            fallback=move || view! {
                <Show when=move || error.get().is_some()>
                    <p class="text-error text-xs">{move || error.get().unwrap_or_default()}</p>
                </Show>
                <p class="text-sm opacity-60">"Loading the status…"</p>
            }
        >
            {move || status.get().map(|found| view! { <ComposeForm initial=Some(found)/> })}
        </Show>
    }
}
