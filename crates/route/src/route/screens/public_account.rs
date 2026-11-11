//! An account a visitor reached from a shared post: its header and its public posts.

use std::sync::Arc;

use leptos::prelude::*;
use nodus_instance::model::Account;

use crate::components::CardRenderer;
use crate::relative_age;

use super::super::Runtime;

#[component]
pub fn PublicAccountScreen(
    runtime: Arc<dyn Runtime>,
    host: String,
    id: String,
    card: CardRenderer,
) -> impl IntoView {
    let runtime = StoredValue::new(runtime);
    let account = RwSignal::new(runtime.get_value().public_account(&host, &id));
    let statuses = RwSignal::new(runtime.get_value().public_account_statuses(&host, &id));
    let error = RwSignal::new(None::<String>);
    let alive = crate::Alive::new();

    let load_host = host.clone();
    let load_id = id.clone();
    Effect::new(move |_| {
        if account.get_untracked().is_some() && !statuses.get_untracked().is_empty() {
            return;
        }
        let done = Callback::new(move |result: Result<Account, String>| {
            alive.if_alive(|| match result {
                Ok(found) => account.set(Some(found)),
                Err(message) => error.set(Some(message)),
            });
        });
        runtime
            .get_value()
            .load_public_account(load_host.clone(), load_id.clone(), done);
        let done = Callback::new(
            move |result: Result<Vec<nodus_instance::model::Status>, String>| {
                alive.if_alive(|| match result {
                    Ok(found) => statuses.set(found),
                    Err(message) => error.set(Some(message)),
                });
            },
        );
        runtime
            .get_value()
            .load_public_account_statuses(load_host.clone(), load_id.clone(), done);
    });

    let age_of = Callback::new(move |created: String| {
        relative_age(&created, runtime.get_value().now_millis())
    });
    let statuses_view = Signal::derive(move || statuses.get());
    let card = StoredValue::new(card);
    let age_of = StoredValue::new(age_of);

    view! {
        <Show
            when=move || account.get().is_some()
            fallback=move || {
                view! {
                    <crate::Screen>
                        <Show when=move || error.get().is_some()>
                            <crate::Note class="text-error text-xs">
                                {move || error.get().unwrap_or_default()}
                            </crate::Note>
                        </Show>
                        <crate::chrome::Skeleton count=4/>
                    </crate::Screen>
                }
                .into_any()
            }
        >
            {move || account.get().map(|profile| view! {
                <crate::profile::ProfilePage
                    account=profile
                    relationship=None
                    statuses=statuses_view
                    show_statuses=true
                    show_verbs=false
                    card=card.get_value()
                    age_of=age_of.get_value()
                />
            })}
        </Show>
    }
}
