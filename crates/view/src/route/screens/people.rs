//! One paged list of accounts.

use std::sync::Arc;

use leptos::prelude::*;
use nodus_instance::model::Account;
use nodus_state::page::{PageEntry, PageItems, PageKind};

use crate::pages::people::{Kind, PeoplePage};

use super::super::{Action, ActionResult, Runtime};

/// The page key a list reads.
pub fn page(kind: Kind, id: &str) -> PageKind {
    match kind {
        Kind::Followers => PageKind::Followers(id.to_owned()),
        Kind::Following => PageKind::Following(id.to_owned()),
        Kind::Mutes => PageKind::Mutes,
        Kind::Blocks => PageKind::Blocks,
    }
}

fn accounts(entry: Option<PageEntry>) -> (Vec<Account>, Option<String>) {
    match entry {
        Some(PageEntry {
            items: PageItems::Accounts(items),
            cursor,
            ..
        }) => (items, cursor),
        _ => (Vec::new(), None),
    }
}

/// One of the four account lists, read through the runtime.
#[component]
pub fn PeopleList(
    runtime: Arc<dyn Runtime>,
    #[prop(into)] id: String,
    kind: Kind,
) -> impl IntoView {
    let runtime = StoredValue::new(runtime);
    let kind = StoredValue::new(kind);
    let id = StoredValue::new(id);
    let key = StoredValue::new(page(kind.get_value(), &id.get_value()));
    let (loading, set_loading) = signal(runtime.get_value().page(&key.get_value()).is_none());
    let error = RwSignal::new(None::<String>);
    let alive = crate::Alive::new();

    let snapshot = Signal::derive(move || {
        runtime.get_value().revision().get();
        accounts(runtime.get_value().page(&key.get_value()))
    });
    let rows = Signal::derive(move || snapshot.get().0);
    let cursor = Signal::derive(move || snapshot.get().1);

    let load = Callback::new(move |after: Option<String>| {
        set_loading.set(true);
        let done = Callback::new(move |result: Result<(), String>| {
            alive.if_alive(|| {
                if let Err(message) = result {
                    error.set(Some(message));
                }
                set_loading.set(false);
            });
        });
        runtime.get_value().load_page(key.get_value(), after, done);
    });

    Effect::new(move |_| {
        if cursor.get_untracked().is_none() && rows.get_untracked().is_empty() {
            load.run(None);
        }
    });

    let on_more = Callback::new(move |after: String| load.run(Some(after)));
    let on_open = Callback::new(move |who: String| {
        runtime.get_value().navigate(format!("/accounts/{who}"));
    });
    let on_undo = Callback::new(move |account: String| {
        let action = match kind.get_value() {
            Kind::Mutes => Action::Mute {
                id: account.clone(),
                on: false,
            },
            Kind::Blocks => Action::Block {
                id: account.clone(),
                on: false,
            },
            Kind::Followers | Kind::Following => return,
        };
        let key = key.get_value();
        let for_done = runtime.get_value();
        let done = Callback::new(move |result: Result<ActionResult, String>| match result {
            Ok(_) => for_done.drop_account(&key, &account),
            Err(message) => error.set(Some(message)),
        });
        runtime.get_value().action(action, done);
    });

    view! {
        <PeoplePage
            kind=kind.get_value()
            accounts=rows
            loading=Signal::derive(move || loading.get())
            error=error.into()
            cursor=cursor
            on_open=on_open
            on_undo=on_undo
            on_more=on_more
        />
    }
}
