//! The compose form, shared by writing and editing.
//!
//! One form for both because the wire draft is the same shape: Mastodon's edit is a `PUT`
//! of the same fields a `POST` takes, so the only difference is the button's label. The
//! page owns the column, the line above the box and the two slots — the writer, and the
//! controls under it. What those controls are, and what they talk to, is the caller's.

use leptos::prelude::*;

use crate::Screen;

pub mod buttons;
pub mod options;

/// The form. The caller mounts the editor; this component places it.
#[component]
pub fn ComposeForm(
    /// "Say something", or "Edit this status".
    label: String,
    /// The writer: the editor the caller mounts, plus anything it hangs beside it.
    body: ChildrenFn,
    /// The settings row, the attachments and the submit row, in that order.
    children: ChildrenFn,
    /// The last send's refusal, when it had one.
    #[prop(optional)]
    error: Option<Signal<Option<String>>>,
) -> impl IntoView {
    // `ChildrenFn` is `Fn`, not `FnOnce`, but it is also not `Copy`; storing it is what lets
    // the closures below call it and stay `Fn` themselves.
    let body = StoredValue::new(body);
    let children = StoredValue::new(children);
    let message = move || error.and_then(|error| error.get());

    view! {
        <Screen>
            <div class="form-control">
                <span class="label-text text-xs opacity-60">{label}</span>
                {body.with_value(|body| body())}
            </div>

            {children.with_value(|children| children())}

            <Show when=move || message().is_some()>
                <p class="text-error text-xs">{move || message().unwrap_or_default()}</p>
            </Show>
        </Screen>
    }
}
