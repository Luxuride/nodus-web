//! One ranked account: who it is and why it is here.
//!
//! The card is the whole product in one unit, so its shapes are shared: an item a
//! reader saw on the server is the same card they get once wasm takes over. What is
//! *not* here is the behaviour — follow, hide and copy are verbs over the SDK, so
//! they arrive as callbacks and the server passes none.

use leptos::prelude::*;

/// Who the account is: avatar, name, handle, and how big it is.
#[component]
#[expect(
    clippy::needless_pass_by_value,
    reason = "a Leptos component receives its props by value"
)]
pub fn Identity(display_name: String, acct: String, avatar_url: String, followers: String) -> impl IntoView {
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
pub fn ReasonBlock(reason: String, hint: String, samples: Option<String>, note: String) -> impl IntoView {
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
