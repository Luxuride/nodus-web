//! The "feature a tag" form.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::state::alive::Alive;
use crate::state::bridge::actions;

/// Turns a typed tag into a featured one.
#[component]
pub fn AddTag(
    busy: ReadSignal<bool>,
    set_busy: WriteSignal<bool>,
    set_error: WriteSignal<Option<String>>,
    on_added: Callback<()>,
) -> impl IntoView {
    let (name, set_name) = signal(String::new());
    let alive = Alive::new();

    let add = move || {
        let tag = name
            .get_untracked()
            .trim()
            .trim_start_matches('#')
            .to_owned();
        if tag.is_empty() {
            return;
        }
        set_busy.set(true);
        set_error.set(None);
        spawn_local(async move {
            match actions::feature_tag(&tag).await {
                Ok(()) => {
                    alive.if_alive(|| {
                        set_name.set(String::new());
                        set_busy.set(false);
                    });
                    on_added.run(());
                }
                Err(message) => alive.if_alive(|| {
                    set_error.set(Some(message));
                    set_busy.set(false);
                }),
            }
        });
    };

    view! {
        <div class="flex gap-2">
            <input
                type="text"
                class="input input-bordered input-sm grow"
                placeholder="rust"
                prop:value=move || name.get()
                on:input=move |event| set_name.set(event_target_value(&event))
            />
            <button
                class="btn btn-primary btn-sm"
                disabled=move || busy.get()
                on:click=move |_| add()
            >
                "Feature"
            </button>
        </div>
    }
}
