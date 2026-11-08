//! The dropdown half of the language picker: a filter field and the languages it matches.
//!
//! Kept apart from the button so neither half outgrows what one screen can read, and so the
//! filter's own state (the query) belongs to the panel rather than to the composer.

use leptos::prelude::*;
use nodus_instance::model::Language;

/// The list under the language button.
#[component]
pub(crate) fn LanguageList(
    /// Every language the instance published.
    options: ReadSignal<Vec<Language>>,
    /// The language currently chosen, marked in the list.
    selected: ReadSignal<String>,
    /// Called with the code the writer picked.
    on_pick: Callback<String>,
) -> impl IntoView {
    let (query, set_query) = signal(String::new());

    let matching = move || {
        let needle = query.get().trim().to_lowercase();
        options
            .get()
            .into_iter()
            .filter(|language| {
                needle.is_empty()
                    || language.name.to_lowercase().contains(&needle)
                    || language.code.to_lowercase().starts_with(&needle)
            })
            .collect::<Vec<_>>()
    };

    view! {
        <div class="absolute left-0 top-full z-50 mt-1 w-64 rounded-box border border-base-300 bg-base-100 p-2 shadow-xl">
            <input
                type="search"
                class="input input-bordered input-sm mb-2 w-full"
                placeholder="Filter languages"
                aria-label="Filter languages"
                prop:value=move || query.get()
                on:input=move |event| set_query.set(event_target_value(&event))
                on:keydown=move |event| {
                    // Enter takes the first match, Escape closes: the two keys a writer presses
                    // without moving their hand off the keyboard.
                    match event.key().as_str() {
                        "Enter" => {
                            if let Some(first) = matching().first() {
                                on_pick.run(first.code.clone());
                            }
                        }
                        "Escape" => on_pick.run(String::new()),
                        _ => {}
                    }
                }
            />
            <ul class="max-h-64 overflow-y-auto" role="listbox">
                <For each=matching key=|language| language.code.clone() let:language>
                    <li>
                        {
                            let code = language.code.clone();
                            let name = language.name.clone();
                            let marked = code.clone();
                            let highlighted = code.clone();
                            view! {
                                <button
                                    type="button"
                                    role="option"
                                    class="flex w-full items-center justify-between gap-2 rounded px-2 py-1 text-left text-sm hover:bg-base-200"
                                    class:bg-base-200=move || highlighted == selected.get()
                                    aria-selected=move || marked == selected.get()
                                    on:click=move |_| on_pick.run(code.clone())
                                >
                                    <span class="truncate">{name}</span>
                                </button>
                            }
                        }
                    </li>
                </For>
            </ul>
        </div>
    }
}
