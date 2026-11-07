//! A post opened from a shared link, with no session.
//!
//! Read-only on purpose: every verb in the card needs a token, and pretending otherwise
//! would offer buttons that cannot work. What a visitor gets is the post, its thread, and
//! one honest sentence about signing in.

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_instance::model::Status;

use super::StatusBody;
use crate::public;
use crate::routes::format::{display_name, relative_time};
use crate::state::alive::Alive;

/// One post and its thread, read anonymously from the instance that hosts it.
#[component]
#[expect(
    clippy::needless_pass_by_value,
    reason = "a Leptos component receives its props by value"
)]
pub fn PublicThread(host: String, id: String) -> impl IntoView {
    let alive = Alive::new();
    let (status, set_status) = signal(None::<Status>);
    let (ancestors, set_ancestors) = signal(Vec::<Status>::new());
    let (replies, set_replies) = signal(Vec::<Status>::new());
    let (error, set_error) = signal(None::<String>);
    let (loading, set_loading) = signal(true);

    let for_post = (host.clone(), id.clone());
    Effect::new(move |_| {
        let (host, id) = for_post.clone();
        spawn_local(async move {
            // Two requests, two outcomes: a thread whose context endpoint refuses still has
            // a post worth showing, so the post decides success and the context is best
            // effort.
            match public::status(&host, &id).await {
                Ok(found) => {
                    alive.if_alive(|| set_status.set(Some(found.clone())));
                    if let Ok(context) = public::context(&host, &id).await {
                        alive.if_alive(|| {
                            set_ancestors.set(context.ancestors);
                            set_replies.set(context.descendants);
                        });
                    }
                }
                Err(failure) => {
                    let message = failure.message();
                    alive.if_alive(|| set_error.set(Some(message)));
                }
            }
            alive.if_alive(|| set_loading.set(false));
        });
    });

    let shown = move || status.get();
    let parents = move || ancestors.get();
    let children = move || replies.get();

    view! {
        <article class="flex flex-col gap-3">
            <p class="text-xs opacity-60">
                {format!("Reading {host} without signing in. Public posts only.")}
            </p>

            <Show when=move || loading.get()>
                <p class="text-sm opacity-60">"Loading the post…"</p>
            </Show>

            <Show when=move || error.get().is_some()>
                <div class="alert alert-warning text-sm">
                    {move || error.get().unwrap_or_default()}
                </div>
            </Show>

            <For each=parents key=|status| status.id.clone() let:status>
                <ReadOnly status=status/>
            </For>

            <Show when=move || shown().is_some()>
                <ReadOnly status=shown().unwrap_or_else(|| unreachable!("guarded by the Show"))/>
            </Show>

            <Show when=move || !children().is_empty()>
                <h2 class="mt-2 text-sm font-medium opacity-70">"Replies"</h2>
                <For each=children key=|status| status.id.clone() let:status>
                    <ReadOnly status=status/>
                </For>
            </Show>

            <p class="mt-2 text-xs opacity-60">
                "Sign in from the home screen to reply, boost or favourite."
            </p>
        </article>
    }
}

/// A post rendered without any action that needs a token.
#[component]
fn ReadOnly(status: Status) -> impl IntoView {
    let (shown, set_shown) = signal(status);
    view! {
        <div class="card bg-base-100 shadow-sm">
            <div class="card-body gap-2 p-3">
                <header class="flex items-center gap-2">
                    <img
                        class="size-8 rounded-full bg-base-300 object-cover"
                        src=move || shown.get().account.avatar.clone()
                        alt=move || shown.get().account.acct.clone()
                        loading="lazy"
                        referrerpolicy="no-referrer"
                    />
                    <div class="min-w-0">
                        <p class="truncate text-sm font-semibold">
                            {move || {
                                let account = shown.get().account;
                                display_name(&account.display_name, &account.acct)
                            }}
                        </p>
                        <p class="truncate text-xs opacity-60">
                            // A remote account's `acct` already carries its domain, and
                            // prefixing that with `@` produces `@@` in the first character
                            // a reader sees.
                            {move || {
                                let account = shown.get().account;
                                let handle = if account.acct.contains('@') {
                                    account.acct.clone()
                                } else {
                                    format!("@{}", account.acct)
                                };
                                format!("{handle} · {}", relative_time(&shown.get().created_at))
                            }}
                        </p>
                    </div>
                </header>
                <StatusBody shown=shown set_shown=set_shown/>
            </div>
        </div>
    }
}
