//! What the account says about itself: display name and bio.

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_pages::pages::settings::form::ProfileFields;

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
    let (fetching, set_fetching) = signal(false);

    Effect::new(move |_| {
        if seeded.get() {
            return;
        }
        if let Some(current) = account() {
            set_display_name.set(current.display_name);
            set_note.set(current.note);
            set_seeded.set(true);
            return;
        }
        if fetching.get_untracked() {
            return;
        }
        let Some(id) = with_sdk(|sdk| sdk.state().session.get().map(|session| session.account_id))
        else {
            return;
        };
        set_fetching.set(true);
        spawn_local(async move {
            let answer = actions::account(&id).await;
            alive.if_alive(|| {
                set_fetching.set(false);
                match answer {
                    Ok(current) => {
                        set_display_name.set(current.display_name);
                        set_note.set(current.note);
                        set_seeded.set(true);
                    }
                    Err(message) => set_message.set(Some(message)),
                }
            });
        });
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

    view! {
        <nodus_ui::components::panel::Panel title="Profile">
            <ProfileFields
                display_name=Signal::derive(move || display_name.get())
                note=Signal::derive(move || note.get())
                max_note=NOTE_MAX_CHARS
                busy=Signal::derive(move || busy.get() || fetching.get())
                message=Signal::derive(move || message.get())
                on_display_name=Some(Callback::new(move |value: String| set_display_name.set(value)))
                on_note=Some(Callback::new(move |value: String| set_note.set(value)))
                on_save=Some(Callback::new(move |()| save()))
            />
        </nodus_ui::components::panel::Panel>
    }
}
