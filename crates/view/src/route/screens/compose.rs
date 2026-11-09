//! Writing and editing, in the same form the browser mounts.
//!
//! The editor is the one piece that cannot exist before wasm: the stateless paint shows
//! the form's own body slot, and the browser replaces it with `ProseMirror` in place.

use std::sync::Arc;

use leptos::prelude::*;

use crate::Note;
use crate::pages::compose::ComposeLoading;
use crate::pages::compose::form::ComposeForm;
use crate::route::Runtime;

#[component]
pub fn ComposeScreen(runtime: Arc<dyn Runtime>, edit: Option<String>) -> impl IntoView {
    let status = edit
        .as_ref()
        .and_then(|id| runtime.status(id));
    let error = Signal::derive(|| None::<String>);

    let body: ChildrenFn = Arc::new(|| {
        view! {
            <div
                id="nodus-composer-body"
                class="min-h-24 w-full cursor-text text-sm leading-relaxed"
                aria-label="Write a post"
            >
                <p class="pointer-events-none opacity-50">"What's on your mind?"</p>
            </div>
        }
        .into_any()
    });
    let controls: ChildrenFn = Arc::new(|| {
        view! { <Note>"The editor needs the app."</Note> }.into_any()
    });

    match (edit, status) {
        (Some(_), None) => view! { <ComposeLoading error=error/> }.into_any(),
        _ => view! {
            <ComposeForm
                label="Say something".to_owned()
                body=body
                children=controls
                error=error
            />
        }
        .into_any(),
    }
}
