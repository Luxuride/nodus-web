//! The form that queues a post.

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_instance::model::Visibility;

use crate::state::alive::Alive;
use crate::state::bridge::actions;

/// Turns a body and a local time into a queued post.
#[component]
pub fn ScheduleForm(
    set_error: WriteSignal<Option<String>>,
    on_queued: Callback<()>,
) -> impl IntoView {
    let (text, set_text) = signal(String::new());
    let (at, set_at) = signal(String::new());
    let (busy, set_busy) = signal(false);
    let alive = Alive::new();

    let queue = move || {
        let body = text.get_untracked();
        let when = at.get_untracked();
        if body.trim().is_empty() || when.is_empty() {
            set_error.set(Some("A scheduled post needs a body and a time.".to_owned()));
            return;
        }
        set_busy.set(true);
        set_error.set(None);
        spawn_local(async move {
            // `datetime-local` hands back local time without a zone; the browser's own
            // `Date` is the only thing that knows the offset, so it does the conversion.
            let millis = js_sys::Date::parse(&when);
            let stamp = if millis.is_nan() {
                when.clone()
            } else {
                String::from(js_sys::Date::new(&millis.into()).to_iso_string())
            };
            match actions::schedule(&body, Visibility::Public, &stamp).await {
                Ok(()) => {
                    alive.if_alive(|| {
                        set_text.set(String::new());
                        set_at.set(String::new());
                        set_busy.set(false);
                    });
                    on_queued.run(());
                }
                Err(message) => alive.if_alive(|| {
                    set_error.set(Some(message));
                    set_busy.set(false);
                }),
            }
        });
    };

    view! {
        <div class="card bg-base-100">
            <div class="card-body gap-2">
                <textarea
                    class="textarea textarea-bordered"
                    rows="3"
                    placeholder="What will you say?"
                    prop:value=move || text.get()
                    on:input=move |event| set_text.set(event_target_value(&event))
                ></textarea>
                <div class="flex flex-wrap items-center gap-2">
                    <input
                        type="datetime-local"
                        class="input input-bordered input-sm"
                        prop:value=move || at.get()
                        on:input=move |event| set_at.set(event_target_value(&event))
                    />
                    <button
                        class="btn btn-primary btn-sm"
                        disabled=move || busy.get()
                        on:click=move |_| queue()
                    >
                        "Schedule"
                    </button>
                </div>
            </div>
        </div>
    }
}
