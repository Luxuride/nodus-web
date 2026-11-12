//! Visibility, content warning and the sensitive flag: the settings row of the composer.
//!
//! The row answers "who may see this" and nothing else. The language picker sits in the
//! same row but arrives as a slot: its list is fetched from the instance, and this crate
//! may not make that request.

use leptos::prelude::*;
use nodus_client_core::model::Visibility;

/// Visibility choices, in the order Mastodon's own picker shows them.
const CHOICES: [Visibility; 4] = [
    Visibility::Public,
    Visibility::Unlisted,
    Visibility::Private,
    Visibility::Direct,
];

/// The label a visibility gets in the picker.
fn label(visibility: Visibility) -> &'static str {
    match visibility {
        Visibility::Public => "Public — anyone can see",
        Visibility::Unlisted => "Unlisted — public, but not in timelines",
        Visibility::Private => "Private — followers only",
        Visibility::Direct => "Direct — mentioned people only",
    }
}

/// The options row and the content-warning field.
#[component]
pub fn ComposerOptions(
    #[prop(into)] visibility: Signal<Visibility>,
    #[prop(into)] warning_open: Signal<bool>,
    #[prop(into)] sensitive: Signal<bool>,
    #[prop(into)] spoiler: Signal<String>,
    /// Picks who may see the post.
    #[prop(optional)]
    on_visibility: Option<Callback<Visibility>>,
    /// Opens or closes the warning row.
    #[prop(optional)]
    on_toggle_warning: Option<Callback<()>>,
    /// Marks the post's media as sensitive.
    #[prop(optional)]
    on_sensitive: Option<Callback<bool>>,
    /// Edits the warning itself.
    #[prop(optional)]
    on_spoiler: Option<Callback<String>>,
    /// The language picker, whose list is the instance's to answer.
    #[prop(optional)]
    language: Option<ChildrenFn>,
) -> impl IntoView {
    view! {
        <div class="flex flex-wrap items-center gap-2">
            <select
                class="select select-bordered select-sm"
                disabled=move || on_visibility.is_none()
                on:change=move |event| {
                    if let Some(on_visibility) = on_visibility {
                        let picked = event_target_value(&event);
                        on_visibility
                            .run(
                                CHOICES
                                    .into_iter()
                                    .find(|choice| choice.as_str() == picked)
                                    .unwrap_or(Visibility::Public),
                            );
                    }
                }
            >
                <For each=|| CHOICES key=|choice| choice.as_str() let:choice>
                    <option value=choice.as_str() selected=move || visibility.try_get().is_some_and(|value| value == choice)>
                        {label(choice)}
                    </option>
                </For>
            </select>

            {language.map(|language| language())}

            <button
                class="btn btn-ghost btn-sm"
                disabled=move || on_toggle_warning.is_none()
                on:click=move |_| {
                    if let Some(on_toggle_warning) = on_toggle_warning {
                        on_toggle_warning.run(());
                    }
                }
            >
                {move || if warning_open.try_get().unwrap_or(false) { "Remove warning" } else { "Add warning" }}
            </button>

            <Show when=move || warning_open.try_get().unwrap_or(false)>
                <label class="label cursor-pointer gap-2">
                    <input
                        class="checkbox checkbox-sm"
                        type="checkbox"
                        disabled=move || on_sensitive.is_none()
                        prop:checked=move || sensitive.get()
                        on:change=move |event| {
                            if let Some(on_sensitive) = on_sensitive {
                                on_sensitive.run(event_target_checked(&event));
                            }
                        }
                    />
                    <span class="label-text text-xs">"Media is sensitive"</span>
                </label>
            </Show>
        </div>

        <Show when=move || warning_open.try_get().unwrap_or(false)>
            <input
                class="input input-bordered input-sm w-full"
                placeholder="Content warning"
                disabled=move || on_spoiler.is_none()
                prop:value=move || spoiler.get()
                on:input=move |event| {
                    if let Some(on_spoiler) = on_spoiler {
                        on_spoiler.run(event_target_value(&event));
                    }
                }
            />
        </Show>
    }
}
