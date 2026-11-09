//! The reader's filters and featured tags: two lists, each with an add form.
//!
//! The card owns the list shape and the empty and error states; the caller supplies the
//! rows and the form, because both carry behaviour this crate cannot know.

use leptos::prelude::*;

use crate::components::panel::Panel;

/// One filter, already reduced to what the row shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilterItem {
    pub id: String,
    pub title: String,
    pub detail: String,
}

/// The filter list, with the form that adds one below it.
#[component]
pub fn FiltersCard(
    #[prop(into)] filters: Signal<Vec<FilterItem>>,
    #[prop(into)] busy: Signal<bool>,
    #[prop(into)] error: Signal<Option<String>>,
    #[prop(optional_no_strip)] on_delete: Option<Callback<String>>,
    add_form: ChildrenFn,
) -> impl IntoView {
    view! {
        <Panel title="Filters">
            <p class="text-sm opacity-70">
                "A filter hides a post, or shows it behind a warning. The instance applies it to timelines it returns."
            </p>

            <Show when=move || error.get().is_some()>
                <p class="text-sm text-error">{move || error.get().unwrap_or_default()}</p>
            </Show>

            <ul class="flex flex-col gap-2">
                {move || {
                    filters
                        .get()
                        .into_iter()
                        .map(|filter| {
                            let id = filter.id;
                            view! {
                                <li class="flex items-start justify-between gap-2 rounded-box bg-base-200 px-3 py-2">
                                    <div class="min-w-0">
                                        <p class="truncate text-sm font-medium">{filter.title}</p>
                                        <p class="truncate text-xs opacity-70">{filter.detail}</p>
                                    </div>
                                    <button
                                        class="btn btn-ghost btn-xs"
                                        disabled=on_delete.is_none()
                                        on:click=move |_| {
                                            if let Some(on_delete) = on_delete {
                                                on_delete.run(id.clone());
                                            }
                                        }
                                    >
                                        "Delete"
                                    </button>
                                </li>
                            }
                        })
                        .collect_view()
                }}
            </ul>

            <Show when=move || filters.get().is_empty() && !busy.get()>
                <p class="text-sm opacity-70">"No filters yet."</p>
            </Show>

            {add_form()}
        </Panel>
    }
}

/// One featured tag.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagItem {
    pub id: String,
    pub label: String,
}

/// The profile's tags, with the form that adds one below them.
#[component]
pub fn FeaturedTagsCard(
    #[prop(into)] tags: Signal<Vec<TagItem>>,
    #[prop(into)] busy: Signal<bool>,
    #[prop(into)] error: Signal<Option<String>>,
    #[prop(optional_no_strip)] on_delete: Option<Callback<String>>,
    add_form: ChildrenFn,
) -> impl IntoView {
    view! {
        <Panel title="Featured tags">
            <p class="text-sm opacity-70">
                "Tags on your profile, so a visitor sees what you post about."
            </p>

            <Show when=move || tags.get().is_empty() fallback=|| ()>
                <p class="text-sm opacity-60">"No featured tags yet."</p>
            </Show>

            <ul class="flex flex-wrap gap-2">
                {move || {
                    tags
                        .get()
                        .into_iter()
                        .map(|tag| {
                            let id = tag.id;
                            view! {
                                <li class="badge badge-lg badge-outline gap-1">
                                    <span>{tag.label}</span>
                                    <button
                                        class="btn btn-ghost btn-xs"
                                        disabled=move || busy.get() || on_delete.is_none()
                                        on:click=move |_| {
                                            if let Some(on_delete) = on_delete {
                                                on_delete.run(id.clone());
                                            }
                                        }
                                    >
                                        "×"
                                    </button>
                                </li>
                            }
                        })
                        .collect_view()
                }}
            </ul>

            {add_form()}

            <Show when=move || error.get().is_some()>
                <p class="text-sm text-error">{move || error.get().unwrap_or_default()}</p>
            </Show>
        </Panel>
    }
}
