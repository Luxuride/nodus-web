//! The feed screen: two lists under one header, one lit at a time.
//!
//! The screen is the shell around two lists the client already owns: people this instance
//! suggests, and posts worth reading. Both stay mounted and are hidden by class, because
//! switching tabs must not throw away the scroll position or an impression that is still
//! being timed; the lit tab is the caller's state, since it is also the screen's address.
//!
//! The header is a slot: the feed is the one screen with a composer above the title, and the
//! composer talks to the SDK, so the client places it and this crate only leaves room.

use leptos::prelude::*;

use crate::{PageHeader, Screen};

pub mod accounts;
pub mod posts;
pub mod reason;

/// The two lists the feed shows.
///
/// The caller holds this because the address holds it too; the page only reads which one is
/// lit and reports presses back.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeedTab {
    People,
    Posts,
}

/// One feed, whatever feeds it: its composer slot, title, tabs, states and list.
///
/// Home, Local, Federated and the recommendation feed all paint this same frame. The
/// caller supplies the rows and the tab strip; the page owns the spacing, the spinner,
/// the error alert and the empty card, so a feed cannot drift from its siblings.
#[component]
#[expect(
    clippy::needless_pass_by_value,
    reason = "a Leptos component receives its props by value"
)]
pub fn FeedPage(
    #[prop(into)] title: String,
    #[prop(into)] description: String,
    #[prop(into)] empty_note: String,
    /// The composer and the feed switch, above the title.
    #[prop(optional_no_strip)]
    header: Option<ChildrenFn>,
    /// An announcement or follow-request banner, above the rows.
    #[prop(optional_no_strip)]
    banner: Option<ChildrenFn>,
    /// Secondary switches beside the title, when the feed has any.
    #[prop(optional_no_strip)]
    tabs: Option<ChildrenFn>,
    #[prop(into)] error: Signal<Option<String>>,
    #[prop(into)] loading: Signal<bool>,
    /// Whether the selected list has rows.
    #[prop(into)]
    has_items: Signal<bool>,
    children: ChildrenFn,
) -> impl IntoView {
    let header = header.map(|header| header());
    let banner = banner.map(|banner| banner());
    let tabs = tabs.map(|tabs| tabs());
    let note = StoredValue::new(empty_note);
    let rows = children();

    view! {
        <Screen>
            <div class="flex flex-col gap-2">{header}</div>

            <div class="mb-5 flex flex-wrap items-end justify-between gap-3">
                <PageHeader title=title description=description/>
                {tabs}
            </div>

            {banner}

            <Show when=move || error.get().is_some()>
                <div class="alert alert-error py-2 text-sm" role="alert">
                    <span>{move || error.get().unwrap_or_default()}</span>
                </div>
            </Show>

            <Show when=move || loading.get() && !has_items.get()>
                <div class="flex justify-center py-6">
                    <span class="loading loading-spinner loading-md"></span>
                </div>
            </Show>

            <Show when=move || {
                !loading.get() && !has_items.get() && error.get().is_none()
            }>
                <div class="card border border-base-300 bg-base-100">
                    <div class="card-body items-center p-6 text-center">
                        <h2 class="card-title text-base">"Nothing here yet"</h2>
                        <p class="text-sm opacity-70">{move || note.try_with_value(Clone::clone).unwrap_or_default()}</p>
                    </div>
                </div>
            </Show>

            {rows}
        </Screen>
    }
}
