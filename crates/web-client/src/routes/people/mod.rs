//! A paged list of accounts: followers, following, mutes and blocks all look like this.
//!
//! The list itself is `nodus_view::pages::people`: the empty line, the rows, the paging
//! control and the error line. What stays here is which page store the list reads, which verb
//! an undo sends, and where a press on a row goes.

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_sdk::Command;
use nodus_sdk::page::PageKind;
use nodus_view::pages::people::PeoplePage;

use crate::router::{self, Route};
use crate::state::alive::Alive;
use crate::state::bridge::{actions, dispatch, pages, revision};

/// The list the view crate draws. Re-exported because the four screens that place one — a
/// profile's followers and following, the viewer's mutes and blocks — name it themselves.
pub use nodus_view::pages::people::Kind;

/// The page store key for a list.
fn page(kind: Kind, id: &str) -> PageKind {
    match kind {
        Kind::Followers => PageKind::Followers(id.to_owned()),
        Kind::Following => PageKind::Following(id.to_owned()),
        Kind::Mutes => PageKind::Mutes,
        Kind::Blocks => PageKind::Blocks,
    }
}

/// One paged list of accounts.
#[component]
pub fn PeopleList(
    #[prop(into)]
    /// Whose list to read. Ignored for mutes and blocks, which are always the viewer's.
    id: String,
    /// Which list this is.
    kind: Kind,
) -> impl IntoView {
    let revision = revision();
    // Asking the store first is what makes a second visit instant: only a list nothing is
    // held for has any reason to look busy.
    let (loading, set_loading) = signal(!pages::held(&page(kind, &id)));
    let (error, set_error) = signal(None::<String>);
    let alive = Alive::new();

    // `StoredValue` rather than a captured `String`: with only `Copy` captures the
    // closure below is `Copy` too, so the effect and the paging control can share it.
    let id = StoredValue::new(id);
    let held = move || {
        revision.get();
        pages::accounts(&page(kind, &id.get_value()))
    };
    let accounts = move || held().map(|(list, _)| list).unwrap_or_default();
    let cursor = move || held().and_then(|(_, cursor)| cursor);

    let load = move |after: Option<String>| {
        let id = id.get_value();

        spawn_local(async move {
            let answer = pages::ensure_after(page(kind, &id), after).await;
            match answer {
                Ok(_) => alive.if_alive(|| set_loading.set(false)),
                Err(message) => alive.if_alive(|| {
                    set_error.set(Some(message));
                    set_loading.set(false);
                }),
            }
        });
    };

    Effect::new(move |_| load(None));

    let on_more = Callback::new(move |after: String| load(Some(after)));
    let on_open = Callback::new(move |who: String| router::navigate(Route::Account(who)));

    let undo = Callback::new(move |account_id: String| {
        spawn_local(async move {
            let answer = match kind {
                Kind::Mutes => actions::mute(&account_id, false).await.map(drop),
                Kind::Blocks => actions::block(&account_id, false).await.map(drop),
                Kind::Followers | Kind::Following => Ok(()),
            };
            match answer {
                Ok(()) => alive.if_alive(|| {
                    // The command owns the page, so the row leaves it and every screen
                    // reading that page agrees.
                    dispatch(Command::PageAccountDropped {
                        kind: page(kind, &id.get_value()),
                        account: account_id,
                    });
                }),
                Err(message) => alive.if_alive(|| set_error.set(Some(message))),
            }
        });
    });

    view! {
        <PeoplePage
            kind=kind
            accounts=Signal::derive(accounts)
            loading=loading.into()
            error=error.into()
            cursor=Signal::derive(cursor)
            on_open=on_open
            on_undo=undo
            on_more=on_more
        />
    }
}
