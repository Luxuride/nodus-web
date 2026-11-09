//! What each address asks the instance for, and how that becomes a page.
//!
//! One function per family of URLs — a list page, an entity (account or thread), the saved
//! shelves, search — so `main.rs` stays a router: it decides *which* reader runs, this module
//! decides what the answer looks like.

use leptos::prelude::{Callback, Owner, Signal};

use crate::mastodon::Client;
use crate::{document, rows::now_millis, PAGE};

/// Renders a component tree to the string the document embeds.
fn painted<V: leptos::IntoView>(view: V) -> String {
    Owner::new().with(|| view.to_html())
}

/// The clock every page's ages are measured against.
fn ages() -> Callback<String, String> {
    Callback::new(|created: String| nodus_view::relative_age(&created, now_millis()))
}

/// A value the server has, handed to a component that expects a signal.
fn held<T: Clone + Send + Sync + 'static>(value: T) -> Signal<T> {
    Signal::derive(move || value.clone())
}

fn empty() -> Signal<Option<String>> {
    Signal::derive(|| None)
}

fn absent<T: Clone + Send + Sync + 'static>() -> Signal<Option<T>> {
    Signal::derive(|| None)
}

fn no() -> Signal<bool> {
    Signal::derive(|| false)
}


pub(crate) async fn read_page(
    client: &Client,
    assets: &document::Assets,
    acct: &str,
    path: &str,
    query: Option<&str>,
) -> Option<String> {
    let page = match path {
        "/notifications" => client.notifications(PAGE).await.map(|items| {
            let inner = painted(nodus_view::pages::notifications::NotificationsPage(
                nodus_view::pages::notifications::NotificationsPageProps {
                    notifications: held(items.clone()),
                    cursor: absent(),
                    loading: no(),
                    error: empty(),
                    age_of: Some(ages()),
                    on_more: None,
                    on_open: None,
                    status_card: None,
                    request_actions: None,
                    banner: None,
                },
            ));
            document::page_of(
                assets,
                acct,
                path,
                "Notifications",
                &inner,
                Some(serde_json::json!({ "kind": "notifications", "items": items, "cursor": null }).to_string()),
            )
        })
        .ok(),
        "/messages" => client.conversations(PAGE).await.map(|items| {
            let inner = painted(nodus_view::pages::messages::MessagesPage(
                nodus_view::pages::messages::MessagesPageProps {
                    conversations: held(items.clone()),
                    muted: held(Vec::new()),
                    loading: no(),
                    error: empty(),
                    cursor: absent(),
                    age_of: Some(ages()),
                    on_open: None,
                    on_read: None,
                    on_mute: None,
                    on_more: None,
                },
            ));
            document::page_of(
                assets,
                acct,
                path,
                "Messages",
                &inner,
                Some(serde_json::json!({ "kind": "conversations", "items": items, "cursor": null }).to_string()),
            )
        })
        .ok(),
        "/saved/favourites" | "/saved/bookmarks" => {
            read_saved(client, assets, acct, path, path.ends_with("bookmarks")).await
        }
        "/lists" => client.lists().await.map(|items| {
            let inner = painted(nodus_view::pages::lists::ListsPage(
                nodus_view::pages::lists::ListsPageProps {
                    lists: held(items.clone()),
                    selected: absent(),
                    title: held(String::new()),
                    busy: no(),
                    error: empty(),
                    on_title: None,
                    on_create: None,
                    row: None,
                    members: None,
                    timeline: None,
                },
            ));
            document::page_of(
                assets,
                acct,
                path,
                "Lists",
                &inner,
                Some(serde_json::json!({ "kind": "lists", "items": items, "cursor": null }).to_string()),
            )
        })
        .ok(),
        "/scheduled" => client.scheduled().await.map(|items| {
            let inner = painted(nodus_view::pages::scheduled::ScheduledPage(
                nodus_view::pages::scheduled::ScheduledPageProps {
                    items: held(items.clone()),
                    loading: no(),
                    error: empty(),
                    age_of: Some(ages()),
                    on_cancel: None,
                },
            ));
            document::page_of(
                assets,
                acct,
                path,
                "Scheduled",
                &inner,
                Some(serde_json::json!({ "kind": "scheduled", "items": items, "cursor": null }).to_string()),
            )
        })
        .ok(),
        "/directory" => client.directory(true).await.map(|items| {
            let inner = painted(nodus_view::pages::directory::DirectoryPage(
                nodus_view::pages::directory::DirectoryPageProps {
                    local_only: Signal::derive(|| true),
                    accounts: held(items.clone()),
                    loading: no(),
                    error: empty(),
                    cursor: absent(),
                    on_scope: None,
                    on_open: None,
                    on_more: None,
                },
            ));
            document::page_of(
                assets,
                acct,
                path,
                "People",
                &inner,
                Some(serde_json::json!({ "kind": "directory", "items": items, "cursor": null }).to_string()),
            )
        })
        .ok(),
        "/search" => read_search(client, assets, acct, path, query).await,
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
    let inner = painted(nodus_view::pages::saved::SavedPage(
        nodus_view::pages::saved::SavedPageProps {
            shelf: Signal::derive(move || shelf),
            items: held(statuses.clone()),
            loading: no(),
            error: empty(),
            cursor: held(cursor.clone()),
            // The server's card is the client's card without its verbs.
            card: std::sync::Arc::new(|status| {
                use leptos::prelude::IntoAny;
                use leptos::tachys::html::element::InnerHtmlAttribute as _;
                leptos::html::div()
                    .inner_html(crate::rows::status_card(&status))
                    .into_any()
            }),
            on_shelf: None,
            on_more: None,
        },
    ));
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
    let inner = painted(nodus_view::pages::search::SearchPage(
        nodus_view::pages::search::SearchPageProps {
            query: held(query.clone()),
            accounts: held(results.accounts.clone()),
            statuses: held(results.statuses.clone()),
            tags: held(results.hashtags.clone()),
            searched: Signal::derive(|| true),
            loading: no(),
            error: empty(),
            on_query: None,
            on_search: None,
            age_of: Some(ages()),
            card: Some(Callback::new(|status| {
                use leptos::prelude::IntoAny;
                use leptos::tachys::html::element::InnerHtmlAttribute as _;
                leptos::html::div()
                    .inner_html(crate::rows::status_card(&status))
                    .into_any()
            })),
        },
    ));
    Some(document::page_of(
        assets,
        acct,
        path,
        "Search",
        &inner,
        None,
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
