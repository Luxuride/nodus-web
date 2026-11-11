//! One ranked account: who it is and why it is here.
//!
//! The card is the whole product in one unit, so its shapes are shared: an item a
//! reader saw on the server is the same card they get once wasm takes over. What is
//! *not* here is the behaviour — follow, hide and copy are verbs over the SDK, so
//! they arrive as callbacks and the server passes none.

use leptos::prelude::*;
use nodus_shared::AccountItem;

/// Who the account is: avatar, name, handle, and how big it is.
#[component]
#[expect(
    clippy::needless_pass_by_value,
    reason = "a Leptos component receives its props by value"
)]
pub fn Identity(
    display_name: String,
    acct: String,
    avatar_url: String,
    followers: String,
) -> impl IntoView {
    view! {
        <div class="flex items-start justify-between gap-3">
            <div class="flex min-w-0 items-center gap-3">
                <div class="avatar">
                    <div class="w-10 rounded-full bg-base-300">
                        <img
                            width="40"
                            height="40"
                            decoding="async"
                            src=avatar_url
                            alt=""
                            referrerpolicy="no-referrer"
                        />
                    </div>
                </div>
                <div class="min-w-0">
                    <p class="truncate font-semibold leading-tight">{display_name}</p>
                    <p class="truncate text-sm opacity-60">{format!("@{acct}")}</p>
                </div>
            </div>
            <span class="badge badge-ghost badge-sm whitespace-nowrap">{followers}</span>
        </div>
    }
}

/// Why the item is on the page: the badge, its hover text, the sample followers,
/// and the account's own description when there is one.
///
/// A badge rather than body text, so it reads as metadata and not as something the
/// account wrote about itself; the hover text carries the raw signal.
#[component]
pub fn ReasonBlock(
    reason: String,
    hint: String,
    samples: Option<String>,
    note: String,
) -> impl IntoView {
    view! {
        <div class="space-y-1">
            <span class="badge badge-soft badge-primary badge-sm" title=hint>{reason}</span>
            {samples.map(|line| view! { <p class="text-xs opacity-70">{line}</p> })}
        </div>
        {(!note.trim().is_empty()).then(|| view! {
            <p class="line-clamp-3 text-sm opacity-80">{note}</p>
        })}
    }
}

/// The card: identity, the reason, the account's note, and the caller's verbs.
///
/// `following` picks the label and the class; `busy` disables the verbs while one is
/// in flight; `toast` is the caller's transient report (the server passes none).
#[component]
#[expect(
    clippy::needless_pass_by_value,
    reason = "a Leptos component receives its props by value"
)]
#[expect(
    clippy::too_many_arguments,
    reason = "the card is one unit: identity, reason, note, and the three verbs with their state"
)]
pub fn AccountCard(
    display_name: String,
    acct: String,
    avatar_url: String,
    followers: String,
    reason: String,
    hint: String,
    samples: Option<String>,
    note: String,
    following: bool,
    busy: Signal<bool>,
    #[prop(optional)] error: Option<Signal<Option<String>>>,
    #[prop(optional)] on_follow: Option<Callback<()>>,
    #[prop(optional)] on_hide: Option<Callback<()>>,
    #[prop(optional)] on_copy: Option<Callback<()>>,
    #[prop(optional)] toast: Option<ChildrenFn>,
) -> impl IntoView {
    let (follow_label, follow_class) = if following {
        ("Following", "btn btn-ghost btn-sm")
    } else {
        ("Follow", "btn btn-primary btn-sm")
    };

    view! {
        <li class="card border border-base-300 bg-base-100 shadow-sm">
            <div class="card-body gap-3 p-4">
                <Identity
                    display_name=display_name
                    acct=acct.clone()
                    avatar_url=avatar_url
                    followers=followers
                />
                <ReasonBlock reason=reason hint=hint samples=samples note=note />
                <div class="card-actions justify-end gap-2">
                    <button
                        class="btn btn-ghost btn-sm"
                        title="Copy profile link"
                        aria-label="Copy profile link"
                        on:click=move |_| {
                            if let Some(copy) = on_copy {
                                copy.run(());
                            }
                        }
                    >
                        "Copy link"
                    </button>
                    <button
                        class="btn btn-ghost btn-sm"
                        disabled=move || busy.get()
                        on:click=move |_| {
                            if let Some(hide) = on_hide {
                                hide.run(());
                            }
                        }
                    >
                        "Not interested"
                    </button>
                    <button
                        class=follow_class
                        disabled=move || busy.get()
                        on:click=move |_| {
                            if let Some(follow) = on_follow {
                                follow.run(());
                            }
                        }
                    >
                        <Show when=move || busy.get() && !following>
                            <span class="loading loading-spinner loading-xs"></span>
                        </Show>
                        {follow_label}
                    </button>
                </div>
                {error
                    .map(|error| {
                        view! {
                            <p class="text-sm text-error" role="alert">
                                {move || error.get().unwrap_or_default()}
                            </p>
                        }
                    })}
                {toast.map(|toast| toast())}
            </div>
        </li>
    }
}

/// The feed's account list: a heading, one refresh verb, the cards, and one more page.
///
/// Every list shape the client had is here: the muted line a weaker ranking deserves, the
/// refusal with its retry, the empty card that explains itself, and the dimmed list while a
/// read is on its way. The cards arrive as a callback because each one carries the reader's
/// follow state, which is the client's to write; the server passes none of the verbs and the
/// list paints the same boxes with its controls disabled.
#[component]
pub fn PeopleList(
    /// The accounts, already in the order they should be read.
    #[prop(into)]
    items: Signal<Vec<AccountItem>>,
    /// The last read's refusal, when it had one.
    #[prop(into)]
    error: Signal<Option<String>>,
    /// Whether a read is already on its way.
    #[prop(into)]
    refreshing: Signal<bool>,
    /// One muted line for a weaker ranking, from `reason::degraded_line`.
    #[prop(into)]
    degraded: Signal<Option<String>>,
    /// The cursor of the next page, when the instance has one.
    #[prop(into)]
    cursor: Signal<Option<String>>,
    /// Paints one card.
    row: Callback<AccountItem, AnyView>,
    /// Asks for the first page again.
    #[prop(optional)]
    on_refresh: Option<Callback<()>>,
    /// Asks for the page after the given cursor.
    #[prop(optional)]
    on_more: Option<Callback<String>>,
    /// A platform trigger for the next page, rendered below the button.
    #[prop(optional_no_strip)]
    auto_more: Option<ChildrenFn>,
    /// Opens the settings screen from the empty card.
    #[prop(optional_no_strip)]
    on_settings: Option<Callback<()>>,
    /// Whether the list paints its own heading, refusal and empty card. A page that owns
    /// those states passes `false` and keeps the same list rhythm.
    #[prop(default = true)]
    show_states: bool,
) -> impl IntoView {
    let row = StoredValue::new(row);
    let refresh_disabled = on_refresh.is_none();
    let more_disabled = on_more.is_none();

    view! {
        <Show when=move || show_states>
        <div class="mb-4 flex items-baseline justify-between gap-4">
            <h2 class="text-sm font-medium opacity-70">"People to follow"</h2>
            <button
                class="btn btn-ghost btn-sm gap-2"
                disabled=move || refreshing.get() || refresh_disabled
                on:click=move |_| {
                    if let Some(on_refresh) = on_refresh {
                        on_refresh.run(());
                    }
                }
            >
                <Show when=move || refreshing.get()>
                    <span class="loading loading-spinner loading-xs"></span>
                </Show>
                "Refresh"
            </button>
        </div>

        {move || {
            degraded
                .get()
                .map(|line| {
                    view! {
                        <div class="alert alert-soft alert-info mb-4 py-2 text-sm" role="status">
                            <span>{line}</span>
                        </div>
                    }
                })
        }}

        {move || {
            error
                .get()
                .map(|message| {
                    view! {
                        <div class="alert alert-soft alert-error mb-4" role="alert">
                            <span>{message}</span>
                            <button
                                class="btn btn-sm"
                                disabled=refresh_disabled
                                on:click=move |_| {
                                    if let Some(on_refresh) = on_refresh {
                                        on_refresh.run(());
                                    }
                                }
                            >
                                "Retry"
                            </button>
                        </div>
                    }
                })
        }}

        <Show when=move || {
            items.get().is_empty() && !refreshing.get() && error.get().is_none()
        }>
            <EmptyPeople on_settings=on_settings/>
        </Show>
        </Show>

        <ul class="space-y-3 transition-opacity" class:opacity-50=move || refreshing.get()>
            <For
                each=move || items.get()
                key=|item| (item.item_id, item.account.already_following)
                let:item
            >
                {row.with_value(|row| row.run(item))}
            </For>
        </ul>

        {move || {
            cursor
                .get()
                .map(|cursor| {
                    view! {
                        <div class="mt-6">
                            <button
                                class="btn btn-outline btn-sm w-full"
                                disabled=move || refreshing.get() || more_disabled
                                on:click=move |_| {
                                    if let Some(on_more) = on_more {
                                        on_more.run(cursor.clone());
                                    }
                                }
                            >
                                {move || {
                                    if refreshing.get() { "Loading…" } else { "Load more people" }
                                }}
                            </button>
                        </div>
                    }
                })
        }}
        {auto_more.map(|more| more())}
    }
}

/// Why the list is empty, and the one thing that changes it.
#[component]
fn EmptyPeople(#[prop(optional_no_strip)] on_settings: Option<Callback<()>>) -> impl IntoView {
    view! {
        <div class="card border border-dashed border-base-300 bg-base-100">
            <div class="card-body items-center gap-2 py-10 text-center">
                <h2 class="card-title text-base">"No recommendations yet"</h2>
                <p class="max-w-sm text-sm opacity-70">
                    "Follow a few accounts to give the recommender something to work with, or turn on background sync in "
                    <a
                        class="link link-primary"
                        href="/settings/account"
                        on:click=move |event| follow_settings(&event, on_settings)
                    >
                        "Settings"
                    </a>
                    " so it can learn from your activity."
                </p>
            </div>
        </div>
    }
}

/// A short-lived report from a verb, placed in the page's toast corner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToastReport {
    pub ok: bool,
    pub message: String,
}

/// The toast corner: one report at a time, green for a success and red for a refusal.
///
/// The slot exists so a card's copy verb can speak without the card knowing where the toast
/// lands. Nothing renders while there is no report.
#[component]
pub fn Toast(#[prop(into)] report: Signal<Option<ToastReport>>) -> impl IntoView {
    view! {
        {move || {
            report
                .get()
                .map(|report| {
                    let tone = if report.ok {
                        "alert alert-success alert-soft text-sm"
                    } else {
                        "alert alert-error alert-soft text-sm"
                    };
                    view! {
                        <div class="toast toast-end" role="status">
                            <div class=tone>{report.message}</div>
                        </div>
                    }
                })
        }}
    }
}

/// Follows the settings anchor when there is a callback for it.
fn follow_settings(event: &leptos::ev::MouseEvent, on_settings: Option<Callback<()>>) {
    #[cfg(not(target_arch = "wasm32"))]
    let _ = event;
    if let Some(on_settings) = on_settings {
        #[cfg(target_arch = "wasm32")]
        event.prevent_default();
        on_settings.run(());
    }
}
