//! The form rows the settings cards are built from.
//!
//! A row owns what it draws and nothing else: the value arrives as a `Signal` and every
//! edit leaves as a `Callback`, so the caller keeps the draft it was already keeping and
//! this crate stays free of storage and transport. Where a callback is absent the control
//! renders disabled rather than pretending to be editable — the server paints the same
//! tree without verbs.
//!
//! The bio counter is here, not in the caller, because it is a display rule: the limit
//! belongs to the instance, and a paste longer than it should be visibly over budget
//! instead of silently truncated.

use leptos::prelude::*;

/// What the account says about itself: display name, bio and the save row.
#[component]
pub fn ProfileFields(
    /// The display name as it is being typed.
    #[prop(into)]
    display_name: Signal<String>,
    /// The bio as it is being typed.
    #[prop(into)]
    note: Signal<String>,
    /// The bio limit the instance enforces.
    max_note: usize,
    /// Whether the save is already on its way.
    #[prop(into)]
    busy: Signal<bool>,
    /// The last save's answer, when there was one.
    #[prop(into)]
    message: Signal<Option<String>>,
    /// Reports the display name as it changes.
    #[prop(optional_no_strip)]
    on_display_name: Option<Callback<String>>,
    /// Reports the bio as it changes.
    #[prop(optional_no_strip)]
    on_note: Option<Callback<String>>,
    /// Saves both fields.
    #[prop(optional_no_strip)]
    on_save: Option<Callback<()>>,
) -> impl IntoView {
    let counter = move || {
        let length = note.get().chars().count();
        let class = if length > max_note {
            "text-xs text-error"
        } else {
            "text-xs opacity-60"
        };
        view! { <span class=class>{format!("{length} / {max_note}")}</span> }
    };

    view! {
        <p class="text-sm opacity-70">"How this account appears to the people who find it."</p>

        <label class="form-control w-full">
            <span class="label-text">"Display name"</span>
            <input
                type="text"
                class="input input-bordered input-sm w-full"
                prop:value=move || display_name.get()
                disabled=on_display_name.is_none()
                on:input=move |event| {
                    if let Some(on_display_name) = on_display_name {
                        on_display_name.run(event_target_value(&event));
                    }
                }
            />
        </label>

        <label class="form-control w-full">
            <span class="label-text">"Bio"</span>
            <textarea
                class="textarea textarea-bordered w-full"
                rows="4"
                prop:value=move || note.get()
                disabled=on_note.is_none()
                on:input=move |event| {
                    if let Some(on_note) = on_note {
                        on_note.run(event_target_value(&event));
                    }
                }
            ></textarea>
            <div class="flex justify-end pt-1">{counter}</div>
        </label>

        <div class="flex items-center gap-3">
            <button
                class="btn btn-primary btn-sm"
                disabled=move || busy.get() || on_save.is_none()
                on:click=move |_| {
                    if let Some(on_save) = on_save {
                        on_save.run(());
                    }
                }
            >
                {move || if busy.get() { "Saving…" } else { "Save profile" }}
            </button>
            <span class="text-sm">{move || message.get().unwrap_or_default()}</span>
        </div>
    }
}

/// Name, phrases and what a match does: the rows of the "add a filter" form.
#[component]
pub fn FilterFields(
    /// The filter's name.
    #[prop(into)]
    name: Signal<String>,
    /// The phrases, as one comma-separated line.
    #[prop(into)]
    phrases: Signal<String>,
    /// Whether a match is hidden instead of warned about.
    #[prop(into)]
    hide: Signal<bool>,
    /// Whether the create is already on its way.
    #[prop(into)]
    busy: Signal<bool>,
    /// Reports the name as it changes.
    #[prop(optional_no_strip)]
    on_name: Option<Callback<String>>,
    /// Reports the phrase line as it changes.
    #[prop(optional_no_strip)]
    on_phrases: Option<Callback<String>>,
    /// Reports the hide choice as it changes.
    #[prop(optional_no_strip)]
    on_hide: Option<Callback<bool>>,
    /// Creates the filter from the current draft.
    #[prop(optional_no_strip)]
    on_add: Option<Callback<()>>,
) -> impl IntoView {
    view! {
        <div class="flex flex-col gap-2">
            <input
                class="input input-bordered input-sm"
                placeholder="Name, for example \"crypto\""
                prop:value=move || name.get()
                disabled=on_name.is_none()
                on:input=move |event| {
                    if let Some(on_name) = on_name {
                        on_name.run(event_target_value(&event));
                    }
                }
            />
            <input
                class="input input-bordered input-sm"
                placeholder="Phrases, separated by commas"
                prop:value=move || phrases.get()
                disabled=on_phrases.is_none()
                on:input=move |event| {
                    if let Some(on_phrases) = on_phrases {
                        on_phrases.run(event_target_value(&event));
                    }
                }
            />
            <label class="label cursor-pointer justify-start gap-2">
                <input
                    type="checkbox"
                    class="toggle toggle-sm"
                    prop:checked=move || hide.get()
                    disabled=on_hide.is_none()
                    on:change=move |event| {
                        if let Some(on_hide) = on_hide {
                            on_hide.run(event_target_checked(&event));
                        }
                    }
                />
                <span class="label-text">"Hide these posts instead of warning"</span>
            </label>
            <button
                class="btn btn-primary btn-sm self-start"
                disabled=move || busy.get() || on_add.is_none()
                on:click=move |_| {
                    if let Some(on_add) = on_add {
                        on_add.run(());
                    }
                }
            >
                {move || if busy.get() { "Working…" } else { "Add filter" }}
            </button>
        </div>
    }
}

/// The row that turns a typed tag into a featured one.
#[component]
pub fn FeaturedTagField(
    /// The tag as it is being typed.
    #[prop(into)]
    name: Signal<String>,
    /// Whether the feature is already on its way.
    #[prop(into)]
    busy: Signal<bool>,
    /// Reports the tag as it changes.
    #[prop(optional_no_strip)]
    on_name: Option<Callback<String>>,
    /// Features the typed tag.
    #[prop(optional_no_strip)]
    on_add: Option<Callback<()>>,
) -> impl IntoView {
    view! {
        <div class="flex gap-2">
            <input
                type="text"
                class="input input-bordered input-sm grow"
                placeholder="rust"
                prop:value=move || name.get()
                disabled=on_name.is_none()
                on:input=move |event| {
                    if let Some(on_name) = on_name {
                        on_name.run(event_target_value(&event));
                    }
                }
            />
            <button
                class="btn btn-primary btn-sm"
                disabled=move || busy.get() || on_add.is_none()
                on:click=move |_| {
                    if let Some(on_add) = on_add {
                        on_add.run(());
                    }
                }
            >
                "Feature"
            </button>
        </div>
    }
}
