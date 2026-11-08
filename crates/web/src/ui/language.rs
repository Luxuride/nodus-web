//! The language picker, shared by every composer.
//!
//! The list comes from the instance, not from a table in this crate: an instance may switch
//! its published languages off or add regional ones, and a picker that offered a code the
//! server does not accept would be a button that always fails.

use leptos::prelude::*;
use nodus_instance::model::Language;

use crate::state::bridge::actions;

/// Chooses the language a post is written in.
///
/// An empty value means "let the instance guess", which is what Mastodon does when the
/// field is absent. The picker starts there so the common case costs no choice.
#[component]
pub fn LanguagePicker(
    language: ReadSignal<String>,
    set_language: WriteSignal<String>,
) -> impl IntoView {
    let (options, set_options) = signal(Vec::<Language>::new());
    let alive = crate::state::alive::Alive::new();

    leptos::task::spawn_local(async move {
        let fetched = actions::languages().await;
        alive.if_alive(|| set_options.set(fetched));
    });

    view! {
        <select
            class="select select-bordered select-sm"
            aria-label="Post language"
            title="The language this post is written in"
            on:change=move |event| set_language.set(event_target_value(&event))
            prop:value=move || language.get()
        >
            <option value="">"Language: instance default"</option>
            <For each=move || options.get() key=|language| language.code.clone() let:language>
                <option value=language.code.clone()>{language.name.clone()}</option>
            </For>
        </select>
    }
}
