//! The reply composer under a thread.
//!
//! The box, the note and the button are shared, so a reader who lands on a painted thread sees
//! the composer where the live one will be. The draft and the request are the client's: this
//! crate receives the text it should show, reports what was typed and what was submitted, and
//! reads no transport.

use leptos::prelude::*;

/// A text box that publishes a public reply to the thread it sits under.
///
/// `draft` is the text the box shows and `posting` disables it while a request is in flight.
/// With no `on_input` the box is read-only and with no `on_submit` the button is disabled, so
/// the paint without a transport keeps the same markup.
#[component]
pub fn ReplyForm(
    /// What the reader has typed so far.
    #[prop(into)]
    draft: Signal<String>,
    /// Whether the reply is already on its way.
    posting: Signal<bool>,
    /// Reports the text as it changes. Absent where the box cannot be typed into.
    #[prop(optional)]
    on_input: Option<Callback<String>>,
    /// Publishes the reply. Absent where there is nowhere to publish it.
    #[prop(optional)]
    on_submit: Option<Callback<()>>,
) -> impl IntoView {
    let unwritable = on_input.is_none();
    let unpublishable = on_submit.is_none();

    view! {
        <form
            class="card border border-base-300 bg-base-100 shadow-sm"
            on:submit=move |event| {
                stop_submit(&event);
                if let Some(on_submit) = on_submit {
                    on_submit.run(());
                }
            }
        >
            <div class="card-body gap-2 p-4">
                <textarea
                    class="textarea textarea-bordered w-full"
                    rows="3"
                    placeholder="Write a reply…"
                    prop:value=move || draft.get()
                    disabled=unwritable
                    on:input=move |event| {
                        if let Some(on_input) = on_input {
                            on_input.run(event_target_value(&event));
                        }
                    }
                ></textarea>
                <div class="flex items-center justify-between gap-2">
                    <span class="text-xs opacity-60">"Public reply"</span>
                    <button
                        class="btn btn-primary btn-sm"
                        type="submit"
                        disabled=move || posting.get() || draft.get().trim().is_empty() || unpublishable
                    >
                        "Reply"
                    </button>
                </div>
            </div>
        </form>
    }
}

/// Keeps a submit from reloading the page, on the target that has a page to reload.
///
/// The server render never runs a listener, and its event type has no `prevent_default`; the
/// non-wasm build keeps a use of `event` so the argument is not reported as unused.
fn stop_submit(event: &leptos::ev::SubmitEvent) {
    #[cfg(not(target_arch = "wasm32"))]
    let _ = event;
    #[cfg(target_arch = "wasm32")]
    event.prevent_default();
}
