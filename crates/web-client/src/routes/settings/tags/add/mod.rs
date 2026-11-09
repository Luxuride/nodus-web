//! The "feature a tag" form.

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_view::pages::settings::form::FeaturedTagField;

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
        <FeaturedTagField
            name=Signal::derive(move || name.get())
            busy=Signal::derive(move || busy.get())
            on_name=Some(Callback::new(move |value: String| set_name.set(value)))
            on_add=Some(Callback::new(move |()| add()))
        />
    }
}
