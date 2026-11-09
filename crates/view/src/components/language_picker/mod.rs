//! The language control: a button that names the chosen language and a filterable list.
//!
//! The catalogue and the chosen code are the caller's; the filter's query belongs to the list
//! because it is not part of the post.

use leptos::prelude::*;
use nodus_instance::model::Language;

/// The language control and its dropdown.
#[component]
pub fn LanguagePicker(
    #[prop(into)] selected: Signal<String>,
    #[prop(into)] name: Signal<String>,
    #[prop(into)] open: Signal<bool>,
    #[prop(into)] options: Signal<Vec<Language>>,
    host: NodeRef<leptos::html::Div>,
    #[prop(optional_no_strip)] on_toggle: Option<Callback<()>>,
    #[prop(optional_no_strip)] on_pick: Option<Callback<String>>,
) -> impl IntoView {
    view! {
        <div class="relative" node_ref=host>
            <button
                type="button"
                class="select select-bordered select-sm w-fit max-w-72 gap-1 bg-none pr-2 text-left"
                aria-haspopup="listbox"
                aria-expanded=move || open.get().to_string()
                aria-label="Post language"
                title="The language this post is written in"
                on:click=move |_| {
                    if let Some(on_toggle) = on_toggle {
                        on_toggle.run(());
                    }
                }
            >
                <span class="whitespace-nowrap">{move || name.get()}</span>
                <span class="text-[0.6rem] leading-none">"▼"</span>
            </button>

            <Show when=move || open.get()>
                <LanguageList options=options selected=selected on_pick=on_pick/>
            </Show>
        </div>
    }
}

/// The list under the language button.
#[component]
fn LanguageList(
    #[prop(into)] options: Signal<Vec<Language>>,
    #[prop(into)] selected: Signal<String>,
    #[prop(optional_no_strip)] on_pick: Option<Callback<String>>,
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

    let pick = move |code: String| {
        if let Some(on_pick) = on_pick {
            on_pick.run(code);
        }
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
                    match event.key().as_str() {
                        "Enter" => {
                            if let Some(first) = matching().first() {
                                pick(first.code.clone());
                            }
                        }
                        "Escape" => pick(String::new()),
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
                            let chosen = code.clone();
                            view! {
                                <button
                                    type="button"
                                    role="option"
                                    class="flex w-full items-center justify-between gap-2 rounded px-2 py-1 text-left text-sm hover:bg-base-200"
                                    class:bg-base-200=move || highlighted == selected.get()
                                    aria-selected=move || marked == selected.get()
                                    on:click=move |_| pick(chosen.clone())
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
