//! A profile: who the account is, what can be done about it, and what they posted.
//!
//! The two paints disagreed about this screen in three ways, and this component settles each
//! one: the header's fields (the server said `followers · following`, the client
//! `following · followers`), the bio (the server drew sanitised HTML, the client the text of
//! it), and the verbs (the server drew none). The first two are one tree now; the third is a
//! callback that is absent on the server, so the button keeps its box and loses only its
//! press.
//!
//! What stays with the screen is what this crate may not know: the tab strip, the cards
//! (a client card has verbs, a media viewer and a clock), the pagination control, and the
//! transport behind each verb. They arrive as slots, and the page owns their order — header,
//! tabs, the status list, then whatever the screen adds under it.

use leptos::prelude::*;
use nodus_instance::model::{Account, Relationship, Status};

use crate::components::CardRenderer;
use crate::display_name;
use crate::status_card::StatusCard;

/// A profile header with Follow, Mute and Block, then the account's posts.
///
/// `show_statuses` hides the list without forgetting that it is empty: an account with no
/// posts and a screen showing another tab are different states, and only the screen knows
/// which one it is in.
#[component]
#[expect(
    clippy::needless_pass_by_value,
    reason = "a Leptos component receives its props by value"
)]
pub fn ProfilePage(
    /// The account this page is about.
    account: Account,
    /// How the reader stands with the account, or `None` while nobody has answered: a verb
    /// without an answer would have to guess "follow" or "unfollow".
    relationship: Option<Relationship>,
    /// The posts, as a signal so the client can append a page without rebuilding the header.
    statuses: Signal<Vec<Status>>,
    /// Whether the status list is the tab on show.
    show_statuses: bool,
    /// The screen's own tab strip, drawn between the header and the list.
    #[prop(optional)]
    tabs: Option<ChildrenFn>,
    /// The screen's own foot: the next page, another tab's list, a transport error.
    #[prop(optional)]
    after: Option<ChildrenFn>,
    /// Paints one status. Absent (the server) means the shared card without verbs.
    #[prop(optional)]
    card: Option<CardRenderer>,
    /// How old a status is, in words, from the caller's clock. Only the default card reads
    /// it; a caller that paints its own cards already owns a clock.
    #[prop(optional)]
    age_of: Option<Callback<String, String>>,
    /// Follows or unfollows; the flag says which. Absent on the server.
    #[prop(optional)]
    on_follow: Option<Callback<(String, bool)>>,
    /// Mutes or unmutes; the flag says which. Absent on the server.
    #[prop(optional)]
    on_mute: Option<Callback<(String, bool)>>,
    /// Blocks or unblocks; the flag says which. Absent on the server.
    #[prop(optional)]
    on_block: Option<Callback<(String, bool)>>,
    /// Opens the picture full size. Absent on the server, where the box is inert.
    #[prop(optional)]
    on_avatar: Option<Callback<String>>,
    /// Whether the reader can act on the relationship at all.
    #[prop(default = true)]
    show_verbs: bool,
) -> impl IntoView {
    let name = display_name(&account.display_name, &account.acct);
    let handle = format!("@{}", account.acct);
    let counts = format!(
        "{} posts · {} following · {} followers",
        account.statuses_count, account.following_count, account.followers_count,
    );
    // The bio is HTML from the instance, already sanitised by the model: drawing it as text
    // would show a reader the tags instead of the links, and `inner_html` is the one honest
    // way to put it in. An empty note draws no paragraph rather than an empty one.
    let bio = account.note.clone();
    let has_bio = !bio.is_empty();
    let header = account.header.clone();
    let has_header = !header.is_empty();
    let avatar = account.avatar.clone();
    let has_avatar = !avatar.is_empty();
    let picture = StoredValue::new(avatar.clone());
    // The id is read by three handlers, so it lives in a stored value: a `String` captured
    // by value would make the first handler `FnOnce`, and a stored value is `Copy`.
    let target = StoredValue::new(account.id.clone());
    // An unanswered relationship promises no verb, so every label says nothing ("…") and
    // every button is disabled. A guessed "Follow" is a claim about state nobody has.
    let ready = relationship.is_some();
    let ((follow, follow_on), (mute, mute_on), (block, block_on)) = match relationship {
        Some(found) => (
            if found.following {
                ("Unfollow", false)
            } else {
                ("Follow", true)
            },
            if found.muting {
                ("Unmute", false)
            } else {
                ("Mute", true)
            },
            if found.blocking {
                ("Unblock", false)
            } else {
                ("Block", true)
            },
        ),
        None => (("…", false), ("…", false), ("…", false)),
    };
    // Stored for the same reason the card stores its slots: the children are taken out of
    // the page once, and a captured `ChildrenFn` would have to be cloned out of `Fn`.
    let tabs = StoredValue::new(tabs);
    let after = StoredValue::new(after);

    let rows = move || statuses.get();
    let custom = card;
    let render = move |status: Status| match custom.as_ref() {
        Some(card) => card(status),
        None => default_card(status, age_of),
    };
    let list = show_statuses.then(|| {
        view! {
            <ul class="flex flex-col gap-3">
                <For each=rows key=|status| status.id.clone() let:status>
                    {render(status)}
                </For>
            </ul>
        }
    });

    view! {
        <crate::Screen>
            <header class="card overflow-hidden border border-base-300 bg-base-100 shadow-sm">
                {has_header.then(|| view! {
                    <img class="h-32 w-full object-cover" src=header alt="" referrerpolicy="no-referrer" />
                })}
                <div class="card-body gap-3 p-4">
                    <div class="flex items-start gap-3">
                        {face(has_avatar, avatar, picture, on_avatar)}
                        <div class="min-w-0 flex-1">
                            <h1 class="truncate text-lg font-semibold">{name}</h1>
                            <p class="truncate text-sm opacity-60">{handle}</p>
                            <p class="mt-2 text-xs opacity-60">{counts}</p>
                        </div>
                    </div>
                    {has_bio
                        .then(|| {
                            view! {
                                <p
                                    class="whitespace-pre-line text-sm leading-relaxed"
                                    inner_html=bio
                                ></p>
                            }
                        })}
                    <Show when=move || show_verbs>
                        <div class="flex flex-wrap items-center gap-2">
                            {verb(follow, "btn btn-primary btn-sm", follow_on, target, on_follow, ready)}
                            {verb(mute, "btn btn-ghost btn-sm", mute_on, target, on_mute, ready)}
                            {verb(block, "btn btn-ghost btn-sm", block_on, target, on_block, ready)}
                        </div>
                    </Show>
                </div>
            </header>

            {tabs.with_value(|tabs| tabs.as_ref().map(|tabs| tabs()))}
            {list}
            {after.with_value(|after| after.as_ref().map(|after| after()))}
        </crate::Screen>
    }
}

/// The profile picture, in the rounded box the account row draws.
///
/// The box is drawn even with no picture, so the names beside it keep their place. Without
/// a callback the box is a `span` with the client's title and none of its affordance: the
/// server paints the same rectangle the client boots in, and a picture that is not there
/// would move the name by twelve units.
fn face(
    has_avatar: bool,
    avatar: String,
    source: StoredValue<String>,
    on_avatar: Option<Callback<String>>,
) -> AnyView {
    let picture = has_avatar.then(|| {
        view! {
            <img
                class="h-16 w-16 rounded-full object-cover"
                width="64"
                height="64"
                decoding="async"
                loading="lazy"
                src=avatar
                alt=""
                referrerpolicy="no-referrer"
            />
        }
    });
    if let Some(on_avatar) = on_avatar {
        return view! {
            <button
                class="avatar cursor-zoom-in"
                type="button"
                title="Expand picture"
                on:click=move |_| on_avatar.run(source.get_value())
            >
                <div class="w-16 rounded-full bg-base-300">{picture}</div>
            </button>
        }
        .into_any();
    }
    view! {
        <span class="avatar cursor-default pointer-events-none" title="Expand picture">
            <div class="w-16 rounded-full bg-base-300">{picture}</div>
        </span>
    }
    .into_any()
}

/// One relationship verb, in the client's box.
///
/// The button is disabled rather than absent where there is no callback or no relationship:
/// the server paints the whole row the client does, and a control that is missing is a row
/// whose width changes when the client boots.
fn verb(
    label: &'static str,
    class: &'static str,
    on: bool,
    target: StoredValue<String>,
    send: Option<Callback<(String, bool)>>,
    ready: bool,
) -> AnyView {
    let disabled = !ready || send.is_none();
    view! {
        <button
            class=class
            disabled=disabled
            on:click=move |_| {
                if let Some(send) = send {
                    send.run((target.get_value(), on));
                }
            }
        >
            {label}
        </button>
    }
    .into_any()
}

/// The card the server paints and the client's fallback: the shared card with no verbs.
///
/// A boost's age belongs to the boosted post, which is the line the card draws.
fn default_card(status: Status, age_of: Option<Callback<String, String>>) -> AnyView {
    let created_at = status
        .reblog
        .as_ref()
        .map_or(status.created_at.as_str(), |inner| {
            inner.created_at.as_str()
        })
        .to_owned();
    let age = age_of.map_or_else(String::new, |age_of| age_of.run(created_at));
    let inner = status.reblog.as_deref().unwrap_or(&status);
    let author_path = format!("/accounts/{}", inner.account.id);
    let thread_path = format!("/status/{}", inner.id);
    view! {
        <StatusCard
            status=status
            age=age
            author_path=author_path
            thread_path=thread_path
            interactive=false
        />
    }
    .into_any()
}
