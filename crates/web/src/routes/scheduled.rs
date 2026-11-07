//! Posts queued for later.
//!
//! Scheduling is the same post with a time attached, so this screen is both the queue and
//! the way to add to it: a small form and the list of what is waiting.

mod form;
mod row;

use leptos::prelude::*;

use crate::ui::page::PageHeader;
use leptos::task::spawn_local;
use nodus_instance::model::ScheduledStatus;

use self::form::ScheduleForm;
use self::row::QueuedRow;
use crate::state::alive::Alive;
use crate::state::bridge::actions;

/// The queue, and a form that adds to it.
#[component]
pub fn Scheduled() -> impl IntoView {
    let (items, set_items) = signal(Vec::<ScheduledStatus>::new());
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);
    let alive = Alive::new();

    let reload = move || {
        let alive = alive;
        spawn_local(async move {
            match actions::scheduled().await {
                Ok(list) => alive.if_alive(|| {
                    set_items.set(list);
                    set_loading.set(false);
                }),
                Err(message) => alive.if_alive(|| {
                    set_error.set(Some(message));
                    set_loading.set(false);
                }),
            }
        });
    };

    Effect::new(move |_| reload());

    let cancel = Callback::new(move |id: String| {
        spawn_local(async move {
            match actions::cancel_scheduled(&id).await {
                Ok(()) => reload(),
                Err(message) => alive.if_alive(|| set_error.set(Some(message))),
            }
        });
    });

    view! {
        <section class="flex flex-col gap-3">
            <PageHeader title="Scheduled" description="Posts waiting to be published, and the form that adds to them."/>

            <ScheduleForm set_error=set_error on_queued=Callback::new(move |()| reload())/>

            {move || error.get().map(|message| view! {
                <div class="alert alert-error py-2 text-sm"><span>{message}</span></div>
            })}

            <Show when=move || loading.get()>
                <p class="text-sm opacity-60">"Loading the queue…"</p>
            </Show>

            <ul class="flex flex-col divide-y divide-base-300 rounded-box bg-base-100">
                <For each=move || items.get() key=|item| item.id.clone() let:item>
                    <QueuedRow item=item on_cancel=cancel/>
                </For>
            </ul>

            <Show when=move || !loading.get() && items.get().is_empty()>
                <p class="text-sm opacity-60">"Nothing is waiting."</p>
            </Show>
        </section>
    }
}
