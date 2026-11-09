//! People this instance suggests.

use std::sync::Arc;

use leptos::prelude::*;
use nodus_state::page::PageKind;

use crate::pages::directory::DirectoryPage;

use super::super::Runtime;

#[component]
pub fn DirectoryScreen(runtime: Arc<dyn Runtime>) -> impl IntoView {
    let revision = runtime.revision();
    let local_only = RwSignal::new(true);
    let loading = RwSignal::new(runtime.page(&PageKind::Directory(true)).is_none());
    let error = RwSignal::new(None::<String>);
    let alive = crate::Alive::new();

    let items = {
        let runtime = Arc::clone(&runtime);
        move || {
            revision.get();
            runtime
                .page(&PageKind::Directory(local_only.get()))
                .map(|page| match page.items {
                    nodus_state::page::PageItems::Accounts(items) => items,
                    _ => Vec::new(),
                })
                .unwrap_or_default()
        }
    };
    let cursor = {
        let runtime = Arc::clone(&runtime);
        move || {
            revision.get();
            runtime
                .page(&PageKind::Directory(local_only.get()))
                .and_then(|page| page.cursor)
        }
    };

    let load: Callback<Option<String>> = Callback::new({
        let runtime = Arc::clone(&runtime);
        move |after: Option<String>| {
            let kind = PageKind::Directory(local_only.get_untracked());
            loading.set(runtime.page(&kind).is_none());
            error.set(None);
            let done = Callback::new(move |result: Result<(), String>| {
                alive.if_alive(|| {
                    loading.set(false);
                    if let Err(message) = result {
                        error.set(Some(message));
                    }
                });
            });
            runtime.load_page(kind, after, done);
        }
    });

    Effect::new(move |_| {
        let _ = local_only.get();
        load.run(None);
    });

    let on_scope = Callback::new(move |here: bool| local_only.set(here));
    let on_open = Callback::new({
        let runtime = Arc::clone(&runtime);
        move |id: String| runtime.navigate(format!("/accounts/{id}"))
    });
    let on_more = Callback::new(move |after: String| load.run(Some(after)));

    view! {
        <DirectoryPage
            local_only=local_only.into()
            accounts=Signal::derive(items)
            loading=loading.into()
            error=error.into()
            cursor=Signal::derive(cursor)
            on_scope=on_scope
            on_open=on_open
            on_more=on_more
        />
    }
}
