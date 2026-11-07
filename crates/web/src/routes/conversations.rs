//! Direct conversations.
//!
//! A conversation is a group of accounts and the last status they sent, which is why the
//! screen shows the last message rather than a message list: opening one is a thread.

mod row;

use leptos::prelude::*;

use crate::ui::page::PageHeader;
use leptos::task::spawn_local;
use nodus_instance::model::Conversation;

use self::row::ConversationRow;
use crate::state::alive::Alive;
use crate::state::bridge::actions;

/// Mutes or unmutes a conversation, and remembers what was asked for.
///
/// The mute state is not part of the conversation payload, so the screen keeps the list
/// of what it muted; the server's answer replaces the row's last message either way.
fn apply_mute(
    id: String,
    mute_now: bool,
    set_items: WriteSignal<Vec<Conversation>>,
    set_muted: WriteSignal<Vec<String>>,
    set_error: WriteSignal<Option<String>>,
    alive: Alive,
) {
    spawn_local(async move {
        match actions::mute_conversation(&id, mute_now).await {
            Ok(updated) => alive.if_alive(|| {
                set_items.update(|held| {
                    if let Some(slot) = held.iter_mut().find(|item| item.id == updated.id) {
                        *slot = updated;
                    }
                });
                set_muted.update(|held| {
                    held.retain(|known| known != &id);
                    if mute_now {
                        held.push(id.clone());
                    }
                });
            }),
            Err(message) => alive.if_alive(|| set_error.set(Some(message))),
        }
    });
}

/// Marks a conversation read; a failure is ignored on purpose.
fn open_conversation(id: String, set_items: WriteSignal<Vec<Conversation>>, alive: Alive) {
    spawn_local(async move {
        let _ = actions::mark_conversation_read(&id).await;
        alive.if_alive(|| {
            set_items.update(|held| {
                if let Some(slot) = held.iter_mut().find(|item| item.id == id) {
                    slot.unread = false;
                }
            });
        });
    });
}

/// The conversations the account is part of.
#[component]
pub fn Conversations() -> impl IntoView {
    let (items, set_items) = signal(Vec::<Conversation>::new());
    let (muted, set_muted) = signal(Vec::<String>::new());
    let (cursor, set_cursor) = signal(None::<String>);
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);
    let alive = Alive::new();

    let load = move |after: Option<String>| {
        let alive = alive;
        spawn_local(async move {
            match actions::conversations(after.as_deref()).await {
                Ok(page) => {
                    let fresh = page.items;
                    alive.if_alive(|| {
                        set_items.update(|held| {
                            for item in fresh {
                                if !held.iter().any(|known| known.id == item.id) {
                                    held.push(item);
                                }
                            }
                        });
                        set_cursor.set(page.next_max_id);
                        set_loading.set(false);
                    });
                }
                Err(message) => alive.if_alive(|| {
                    set_error.set(Some(message));
                    set_loading.set(false);
                }),
            }
        });
    };

    Effect::new(move |_| load(None));
    let on_mute = Callback::new(move |(id, mute_now): (String, bool)| {
        apply_mute(id, mute_now, set_items, set_muted, set_error, alive);
    });
    let on_open = Callback::new(move |id: String| open_conversation(id, set_items, alive));

    view! {
        <section class="flex flex-col gap-3">
            <PageHeader title="Messages" description="Direct conversations with other accounts. Opening one marks it read."/>

            <Show when=move || loading.get()>
                <p class="text-sm opacity-60">"Loading conversations…"</p>
            </Show>
            {move || error.get().map(|message| view! {
                <div class="alert alert-error py-2 text-sm"><span>{message}</span></div>
            })}

            <ul class="flex flex-col divide-y divide-base-300 rounded-box bg-base-100">
                <For each=move || items.get() key=|item| item.id.clone() let:item>
                    {
                        let is_muted = muted.get().contains(&item.id);
                        view! {
                            <ConversationRow
                                item=item
                                is_muted=is_muted
                                on_open=on_open
                                on_mute=on_mute
                            />
                        }
                    }
                </For>
            </ul>

            <Show when=move || !loading.get() && items.get().is_empty() && error.get().is_none()>
                <p class="text-sm opacity-60">"No conversations yet."</p>
            </Show>
            <Show when=move || cursor.get().is_some()>
                <button
                    class="btn btn-sm self-start"
                    on:click=move |_| load(cursor.get_untracked())
                >
                    "Load more"
                </button>
            </Show>
        </section>
    }
}
