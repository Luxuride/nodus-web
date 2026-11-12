//! The form that adds a filter.

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_client_core::model::{FilterAction, FilterDraft};
use nodus_pages::pages::settings::form::FilterFields;

use crate::state::alive::Alive;
use crate::state::bridge::actions;

/// Name, phrases and what a match does.
///
/// Owns its own draft: the card above only needs to know that one was created, and a
/// half-typed phrase must not be re-rendered by an unrelated update to the list.
#[component]
pub fn NewFilterForm(
    on_created: Callback<()>,
    set_error: WriteSignal<Option<String>>,
) -> impl IntoView {
    let alive = Alive::new();
    let (title, set_title) = signal(String::new());
    let (phrases, set_phrases) = signal(String::new());
    let (hide, set_hide) = signal(false);
    let (busy, set_busy) = signal(false);

    let submit = move |()| {
        let name = title.get_untracked().trim().to_owned();
        if name.is_empty() {
            set_error.set(Some("Give the filter a name.".to_owned()));
            return;
        }
        let keywords: Vec<String> = phrases
            .get_untracked()
            .split(',')
            .map(|phrase| phrase.trim().to_owned())
            .filter(|phrase| !phrase.is_empty())
            .collect();
        let action = if hide.get_untracked() {
            FilterAction::Hide
        } else {
            FilterAction::Warn
        };
        let draft = FilterDraft::new(name).action(action).keywords(keywords);

        set_busy.set(true);
        spawn_local(async move {
            match actions::create_filter(&draft).await {
                Ok(_) => {
                    alive.if_alive(|| {
                        set_title.set(String::new());
                        set_phrases.set(String::new());
                        set_error.set(None);
                        set_busy.set(false);
                    });
                    on_created.run(());
                }
                Err(message) => alive.if_alive(|| {
                    set_error.set(Some(message));
                    set_busy.set(false);
                }),
            }
        });
    };

    view! {
        <FilterFields
            name=Signal::derive(move || title.get())
            phrases=Signal::derive(move || phrases.get())
            hide=Signal::derive(move || hide.get())
            busy=Signal::derive(move || busy.get())
            on_name=Some(Callback::new(move |value: String| set_title.set(value)))
            on_phrases=Some(Callback::new(move |value: String| set_phrases.set(value)))
            on_hide=Some(Callback::new(move |value: bool| set_hide.set(value)))
            on_add=Some(Callback::new(move |()| submit(())))
        />
    }
}
