//! The reader's filters: phrases that hide a post, or put it behind a warning.

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_instance::model::Filter;

mod new;
mod row;

use self::new::NewFilterForm;
use self::row::FilterRow;

use crate::state::alive::Alive;
use crate::state::bridge::actions;

/// Settings card listing filters and offering to add one.
///
/// The server applies filters to timelines it returns; this screen is the editor, so the
/// list here is the account's own record rather than a local guess.
#[component]
pub fn FiltersCard() -> impl IntoView {
    let alive = Alive::new();
    let (filters, set_filters) = signal(Vec::<Filter>::new());
    let (busy, set_busy) = signal(true);
    let (error, set_error) = signal(None::<String>);

    let reload = Callback::new(move |()| {
        spawn_local(async move {
            let outcome = actions::filters().await;
            alive.if_alive(|| {
                set_busy.set(true);
                match outcome {
                    Ok(list) => {
                        set_filters.set(list);
                        set_error.set(None);
                    }
                    Err(message) => set_error.set(Some(message)),
                }
                set_busy.set(false);
            });
        });
    });

    Effect::new(move |_| reload.run(()));

    view! {
        <section class="card bg-base-100 shadow-sm">
            <div class="card-body gap-3">
                <h2 class="card-title text-base">"Filters"</h2>
                <p class="text-sm opacity-70">
                    "A filter hides a post, or shows it behind a warning. The instance applies \
                     it to timelines it returns."
                </p>

                <Show when=move || error.get().is_some()>
                    <p class="text-sm text-error">{move || error.get().unwrap_or_default()}</p>
                </Show>

                <ul class="flex flex-col gap-2">
                    <For each=move || filters.get() key=|filter| filter.id.clone() let:filter>
                        <FilterRow filter=filter reload=reload/>
                    </For>
                </ul>

                <Show when=move || filters.get().is_empty() && !busy.get()>
                    <p class="text-sm opacity-70">"No filters yet."</p>
                </Show>

                <NewFilterForm on_created=Callback::new(move |()| reload.run(())) set_error=set_error/>
            </div>
        </section>
    }
}
