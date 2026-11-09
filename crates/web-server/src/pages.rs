//! What each address asks the instance for, and how that becomes a page.
//!
//! One function per family of URLs — a list page, an entity (account or thread), the saved
//! shelves, search — so `main.rs` stays a router: it decides *which* reader runs, this module
//! decides what the answer looks like.

use leptos::prelude::Owner;

use nodus_web::shared::screens::{Screen, ScreenKind};

use crate::mastodon::Client;
use crate::{PAGE, document, rows::now_millis};

/// The server's card painter: it has a status and needs the component's HTML.
fn card() -> nodus_view::components::card_renderer::CardRenderer {
    std::sync::Arc::new(|status: nodus_instance::model::Status| card_view_for(&status))
}

/// The same painter where a page asks for a callback rather than a renderer.
fn card_callback()
-> leptos::prelude::Callback<nodus_instance::model::Status, leptos::prelude::AnyView> {
    leptos::prelude::Callback::new(|status: nodus_instance::model::Status| card_view_for(&status))
}

fn card_view_for(status: &nodus_instance::model::Status) -> leptos::prelude::AnyView {
    use leptos::prelude::IntoAny;
    use leptos::tachys::html::element::InnerHtmlAttribute as _;

    leptos::html::div()
        .inner_html(crate::rows::status_card(status))
        .into_any()
}

/// Renders a component tree to the string the document embeds.
///
/// The closure is not a convenience: a component's body runs when it is *called*, and a
/// `For` inside that body asks `Owner::current()` while it renders. Building the props and
/// the component inside `Owner::new().with(..)` is what keeps that answer non-empty —
/// `painted(Page(props))` would build the page before the owner exists and panic with
/// "no reactive owner" the first time a list had a row in it.
fn painted<V: leptos::IntoView>(view: impl FnOnce() -> V) -> String {
    Owner::new().with(|| view().to_html())
}

pub(crate) async fn read_page(
    client: &Client,
    assets: &document::Assets,
    acct: &str,
    path: &str,
    query: Option<&str>,
) -> Option<String> {
    let Some(screen) = Screen::of_path(path) else {
        return read_entity(client, assets, acct, path).await;
    };

    let page = match screen.kind {
        ScreenKind::Notifications => client.notifications(PAGE).await.map(|items| {
            let inner = painted(|| nodus_view::pages::notifications::NotificationsPage(nodus_view::pages::notifications::NotificationsPageProps::server(items.clone(), Some(nodus_view::components::props::ages(now_millis())), None)));
            document::page_of(
                assets,
                acct,
                path,
                screen.title,
                &inner,
                Some(serde_json::json!({ "kind": "notifications", "items": items, "cursor": null }).to_string()),
            )
        })
        .ok(),
        ScreenKind::Messages => client.conversations(PAGE).await.map(|items| {
            let inner = painted(|| nodus_view::pages::messages::MessagesPage(nodus_view::pages::messages::MessagesPageProps::server(items.clone(), Some(nodus_view::components::props::ages(now_millis())))));
            document::page_of(
                assets,
                acct,
                path,
                screen.title,
                &inner,
                Some(serde_json::json!({ "kind": "conversations", "items": items, "cursor": null }).to_string()),
            )
        })
        .ok(),
        ScreenKind::Saved { bookmarks } => read_saved(client, assets, acct, path, bookmarks).await,
        ScreenKind::Lists => client.lists().await.map(|items| {
            let inner = painted(|| nodus_view::pages::lists::ListsPage(nodus_view::pages::lists::ListsPageProps::server(items.clone())));
            document::page_of(
                assets,
                acct,
                path,
                screen.title,
                &inner,
                Some(serde_json::json!({ "kind": "lists", "items": items, "cursor": null }).to_string()),
            )
        })
        .ok(),
        ScreenKind::Scheduled => client.scheduled().await.map(|items| {
            let inner = painted(|| nodus_view::pages::scheduled::ScheduledPage(nodus_view::pages::scheduled::ScheduledPageProps::server(items.clone(), Some(nodus_view::components::props::ages(now_millis())))));
            document::page_of(
                assets,
                acct,
                path,
                screen.title,
                &inner,
                Some(serde_json::json!({ "kind": "scheduled", "items": items, "cursor": null }).to_string()),
            )
        })
        .ok(),
        ScreenKind::Directory => client.directory(true).await.map(|items| {
            let inner = painted(|| nodus_view::pages::directory::DirectoryPage(nodus_view::pages::directory::DirectoryPageProps::server(items.clone())));
            document::page_of(
                assets,
                acct,
                path,
                screen.title,
                &inner,
                Some(serde_json::json!({ "kind": "directory", "items": items, "cursor": null }).to_string()),
            )
        })
        .ok(),
        ScreenKind::Search => read_search(client, assets, acct, path, query).await,
        _ => None,
    };
    if page.is_some() {
        return page;
    }
    read_entity(client, assets, acct, path).await
}

/// Before the routes with a name in them: one account, or one thread.
pub(crate) async fn read_entity(
    client: &Client,
    assets: &document::Assets,
    acct: &str,
    path: &str,
) -> Option<String> {
    if let Some(id) = path.strip_prefix("/accounts/") {
        if !id.is_empty() && !id.contains('/') {
            let account = client.account(id).await.ok()?;
            let (statuses, cursor) = client.account_statuses(id, PAGE).await.ok()?;
            return Some(document::profile_page(
                assets,
                acct,
                &account,
                &statuses,
                cursor.as_deref(),
            ));
        }
        return None;
    }
    let id = path.strip_prefix("/status/")?;
    if id.is_empty() || id.contains('/') {
        return None;
    }
    let status = client.status(id).await.ok()?;
    let context = client.context(id).await.ok()?;
    Some(document::thread_page(assets, acct, &status, &context))
}

/// Favourites or bookmarks: one screen, two shelves, the same shape.
pub(crate) async fn read_saved(
    client: &Client,
    assets: &document::Assets,
    acct: &str,
    path: &str,
    bookmarks: bool,
) -> Option<String> {
    use nodus_view::pages::saved::shelf::Shelf;

    let (statuses, cursor) = client.saved(bookmarks, PAGE).await.ok()?;
    let shelf = if bookmarks {
        Shelf::Bookmarks
    } else {
        Shelf::Favourites
    };
    let inner = painted(|| {
        nodus_view::pages::saved::SavedPage(nodus_view::pages::saved::SavedPageProps::server(
            shelf,
            statuses.clone(),
            card(),
        ))
    });
    Some(document::page_of(
        assets,
        acct,
        path,
        shelf.label(),
        &inner,
        Some(
            serde_json::json!({
                "kind": if bookmarks { "bookmarks" } else { "favourites" },
                "items": statuses,
                "cursor": cursor,
            })
            .to_string(),
        ),
    ))
}

/// The search page, the one route whose argument comes from the query string.
pub(crate) async fn read_search(
    client: &Client,
    assets: &document::Assets,
    acct: &str,
    path: &str,
    query: Option<&str>,
) -> Option<String> {
    let query = query
        .and_then(|query| {
            query.split('&').find_map(|pair| {
                let (name, value) = pair.split_once('=')?;
                (name == "q").then(|| percent_decode(value))
            })
        })
        .unwrap_or_default();
    if query.trim().is_empty() {
        return None;
    }
    let results = client.search(&query).await.ok()?;
    let inner = painted(|| {
        nodus_view::pages::search::SearchPage(nodus_view::pages::search::SearchPageProps::server(
            query.clone(),
            results.clone(),
            Some(nodus_view::components::props::ages(now_millis())),
            Some(card_callback()),
        ))
    });
    let island = serde_json::json!({
        "kind": "search",
        "query": query,
        "items": results,
        "cursor": null,
    })
    .to_string();
    Some(document::page_of(
        assets,
        acct,
        path,
        "Search",
        &inner,
        Some(island),
    ))
}

/// Percent-decoding for the one query parameter this host reads. `+` is a space because a
/// search box is what sends it.
pub(crate) fn percent_decode(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'+' => {
                decoded.push(b' ');
                index += 1;
            }
            b'%' if index + 2 < bytes.len() => {
                let hex = std::str::from_utf8(&bytes[index + 1..index + 3]).ok();
                if let Some(byte) = hex.and_then(|hex| u8::from_str_radix(hex, 16).ok()) {
                    decoded.push(byte);
                    index += 3;
                } else {
                    decoded.push(bytes[index]);
                    index += 1;
                }
            }
            byte => {
                decoded.push(byte);
                index += 1;
            }
        }
    }
    String::from_utf8_lossy(&decoded).into_owned()
}

#[cfg(test)]
mod painting {
    use leptos::prelude::{ElementChild as _, Get as _};

    use super::painted;

    /// A page built *outside* the owner panics the moment a list has a row in it:
    /// `panicked at leptos/src/for_loop.rs:138: no reactive owner`. Building it inside
    /// `painted`'s closure is the fix, and this test is the shape of that fix.
    #[test]
    fn a_for_with_a_row_is_built_inside_the_owner() {
        let html = painted(|| {
            let items = leptos::prelude::Signal::derive(|| vec![1_u8, 2]);
            leptos::prelude::IntoView::into_view(
                leptos::html::ul().child(leptos::prelude::For(
                    leptos::prelude::ForProps::builder()
                        .each(move || items.get())
                        .key(|value: &u8| *value)
                        .children(|value: u8| leptos::html::li().child(value.to_string()))
                        .build(),
                )),
            )
        });

        assert!(html.contains("<li>1</li>"), "{html}");
    }
}
