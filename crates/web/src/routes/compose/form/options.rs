//! Visibility, content warning and the sensitive flag: the settings row of the composer.

use leptos::prelude::*;
use nodus_instance::model::Visibility;

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
    visibility: ReadSignal<Visibility>,
    set_visibility: WriteSignal<Visibility>,
    warning_open: ReadSignal<bool>,
    set_warning_open: WriteSignal<bool>,
    sensitive: ReadSignal<bool>,
    set_sensitive: WriteSignal<bool>,
    spoiler: ReadSignal<String>,
    set_spoiler: WriteSignal<String>,
    language: ReadSignal<String>,
    set_language: WriteSignal<String>,
) -> impl IntoView {
    view! {
        <div class="flex flex-wrap items-center gap-2">
            <select
                class="select select-bordered select-sm"
                on:change=move |event| {
                    let picked = event_target_value(&event);
                    set_visibility.set(
                        CHOICES
                            .into_iter()
                            .find(|choice| choice.as_str() == picked)
                            .unwrap_or(Visibility::Public),
                    );
                }
            >
                <For each=|| CHOICES key=|choice| choice.as_str() let:choice>
                    <option value=choice.as_str() selected=move || visibility.get() == choice>
                        {label(choice)}
                    </option>
                </For>
            </select>

            <crate::ui::language::LanguagePicker language=language set_language=set_language/>

            <button
                class="btn btn-ghost btn-sm"
                on:click=move |_| set_warning_open.update(|open| *open = !*open)
            >
                {move || if warning_open.get() { "Remove warning" } else { "Add warning" }}
            </button>

            <Show when=move || warning_open.get()>
                <label class="label cursor-pointer gap-2">
                    <input
                        class="checkbox checkbox-sm"
                        type="checkbox"
                        prop:checked=move || sensitive.get()
                        on:change=move |event| set_sensitive.set(event_target_checked(&event))
                    />
                    <span class="label-text text-xs">"Media is sensitive"</span>
                </label>
            </Show>
        </div>

        <Show when=move || warning_open.get()>
            <input
                class="input input-bordered input-sm w-full"
                placeholder="Content warning"
                prop:value=move || spoiler.get()
                on:input=move |event| set_spoiler.set(event_target_value(&event))
            />
        </Show>
    }
}
