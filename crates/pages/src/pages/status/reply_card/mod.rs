//! The box under a thread that writes a reply.
//!
//! The same card appears whether the browser or a server paints it: a border, a textarea
//! and one button. What is not here is publishing — that is a verb over the SDK, so it
//! arrives as a callback and the server passes none, which leaves the card drawn and the
//! button inert rather than a different shape.

use leptos::prelude::*;

/// A bordered card with a draft in it.
///
/// `text` is the draft as it last stood. The component does not own it: a typing reader
/// sends `on_input` and the parent stores the answer, which is what keeps the draft alive
/// across a repaint (and is why the server can pass a fixed `text`).
#[component]
pub fn ReplyCard(
    text: String,
    busy: bool,
    #[prop(optional)] on_input: Option<Callback<String>>,
    #[prop(optional)] on_submit: Option<Callback<()>>,
) -> impl IntoView {
    let empty = text.trim().is_empty();

    view! {
        <form
            class="card border border-base-300 bg-base-100 shadow-sm"
            on:submit=move |event| {
                event.prevent_default();
                if let Some(submit) = on_submit {
                    submit.run(());
                }
            }
        >
            <div class="card-body gap-2 p-3 sm:p-4">
                <textarea
                    class="textarea textarea-bordered min-h-20 w-full resize-none"
                    placeholder="Write a reply…"
                    aria-label="Write a reply"
                    disabled=busy
                    prop:value=text
                    on:input=move |event| {
                        if let Some(input) = on_input {
                            input.run(event_target_value(&event));
                        }
                    }
                ></textarea>
                <div class="flex items-center justify-between gap-2">
                    <span class="text-xs opacity-60">"Public reply"</span>
                    <button
                        class="btn btn-primary btn-sm h-9 min-h-9 px-4"
                        type="submit"
                        disabled=busy || empty
                    >
                        "Reply"
                    </button>
                </div>
            </div>
        </form>
    }
}
