//! What the account says about itself: display name and bio.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::state::alive::Alive;
use crate::state::bridge::{actions, revision, with_sdk};

/// The bio limit the instance enforces. Shown as a counter rather than enforced on every
/// keystroke: the instance owns the rule, and a paste longer than it should be visibly
/// over budget instead of silently truncated here.
const NOTE_MAX_CHARS: usize = 500;

/// The profile card.
///
/// The fields are seeded once and then owned by the user until Save: writing them on every
/// store change would overwrite what is being typed as soon as any other screen refreshed
/// the account.
#[component]
pub(crate) fn ProfileCard() -> impl IntoView {
    let revision = revision();
    let alive = Alive::new();

    let account = move || {
        revision.get();
        with_sdk(|sdk| {
            sdk.state()
                .session
                .get()
                .and_then(|session| sdk.read_account(&session.account_id))
        })
    };

    let (display_name, set_display_name) = signal(String::new());
    let (note, set_note) = signal(String::new());
    let (busy, set_busy) = signal(false);
    let (message, set_message) = signal(None::<String>);
    let (seeded, set_seeded) = signal(false);

    Effect::new(move |_| {
        if seeded.get() {
            return;
        }
        let Some(current) = account() else {
            return;
        };
        set_display_name.set(current.display_name);
        set_note.set(current.note);
        set_seeded.set(true);
    });

    let save = move || {
        if busy.get() {
            return;
        }
        set_busy.set(true);
        set_message.set(None);
        let display_name = display_name.get_untracked();
        let note = note.get_untracked();
        spawn_local(async move {
            let answer = actions::update_profile(display_name, note).await;
            alive.if_alive(|| {
                set_busy.set(false);
                set_message.set(Some(match answer {
                    Ok(_) => "Saved.".to_owned(),
                    Err(message) => message,
                }));
            });
        });
    };

    let counter = move || {
        let length = note.get().chars().count();
        let class = if length > NOTE_MAX_CHARS {
            "text-xs text-error"
        } else {
            "text-xs opacity-60"
        };
        view! { <span class=class>{format!("{length} / {NOTE_MAX_CHARS}")}</span> }
    };

    view! {
        <nodus_view::components::panel::Panel title="Profile">
                <p class="text-sm opacity-70">
                    "How this account appears to the people who find it."
                </p>

                <label class="form-control w-full">
                    <span class="label-text">"Display name"</span>
                    <input
                        type="text"
                        class="input input-bordered input-sm w-full"
                        prop:value=move || display_name.get()
                        on:input=move |event| set_display_name.set(event_target_value(&event))
                    />
                </label>

                <label class="form-control w-full">
                    <span class="label-text">"Bio"</span>
                    <textarea
                        class="textarea textarea-bordered w-full"
                        rows="4"
                        prop:value=move || note.get()
                        on:input=move |event| set_note.set(event_target_value(&event))
                    ></textarea>
                    <div class="flex justify-end pt-1">{counter}</div>
                </label>

                <div class="flex items-center gap-3">
                    <button
                        class="btn btn-primary btn-sm"
                        disabled=move || busy.get()
                        on:click=move |_| save()
                    >
                        {move || if busy.get() { "Saving…" } else { "Save profile" }}
                    </button>
                    <span class="text-sm">{move || message.get().unwrap_or_default()}</span>
                </div>
        </nodus_view::components::panel::Panel>
    }
}
