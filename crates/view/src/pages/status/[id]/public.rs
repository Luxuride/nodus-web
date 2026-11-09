//! A post opened from a shared link, with no session.
//!
//! The document is read-only on purpose: every verb in the card needs a token, and pretending
//! otherwise would offer buttons that cannot work. What a visitor gets is the post, its thread,
//! and one honest sentence about signing in.
//!
//! Two screens live here because the address decides between them: an address with no host has
//! nowhere to read from, so it asks for one; once there is a host, the caller's two reads fill
//! the article and this file paints it. Nothing here reads a URL, a clock or a transport.

use leptos::prelude::*;
use nodus_instance::model::Status;

use crate::display_name;
use crate::status_body::{StatusBody, StatusBodyProps};

/// Asks which instance a host-less link should be read from.
///
/// The form travels with the reader: the id is local to an instance, so guessing one would
/// open the wrong post. `typed` is the host as it has been typed; with no callbacks the field
/// and the button are still there, disabled.
#[component]
pub fn HostForm(
    /// What the reader has typed into the host field.
    #[prop(into)]
    typed: Signal<String>,
    /// Reports the host as it changes.
    #[prop(optional)]
    on_input: Option<Callback<String>>,
    /// Reads the post from the typed host.
    #[prop(optional)]
    on_read: Option<Callback<()>>,
) -> impl IntoView {
    let unreadable = on_read.is_none();

    view! {
        <form
            class="card bg-base-100 shadow-sm"
            on:submit=move |event| {
                stop_submit(&event);
                if let Some(on_read) = on_read {
                    on_read.run(());
                }
            }
        >
            <div class="card-body gap-3">
                <h1 class="card-title text-base">"Read this post"</h1>
                <p class="text-sm opacity-70">
                    "A post id only means something on the instance that issued it. Name the instance to read from."
                </p>
                <input
                    class="input input-sm w-full"
                    placeholder="mastodon.example"
                    prop:value=move || typed.get()
                    disabled=on_input.is_none()
                    on:input=move |event| {
                        if let Some(on_input) = on_input {
                            on_input.run(event_target_value(&event));
                        }
                    }
                />
                <button class="btn btn-primary btn-sm self-end" type="submit" disabled=unreadable>
                    "Read post"
                </button>
            </div>
        </form>
    }
}

/// One post and its thread, read anonymously from the instance that hosts it.
#[component]
pub fn PublicThreadPage(
    /// The instance the post is read from.
    host: String,
    /// The post, once it has arrived.
    #[prop(into)]
    status: Signal<Option<Status>>,
    /// The statuses above it, oldest first.
    #[prop(into)]
    ancestors: Signal<Vec<Status>>,
    /// The answers under it.
    #[prop(into)]
    replies: Signal<Vec<Status>>,
    /// Whether the post is still on its way.
    loading: Signal<bool>,
    /// The last refusal, when the read had one.
    error: Signal<Option<String>>,
    /// How old a status is, in words, from the caller's clock.
    age_of: Callback<String, String>,
) -> impl IntoView {
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

            <For each=move || ancestors.get() key=|status| status.id.clone() let:status>
                <ReadOnly status=status age_of=age_of/>
            </For>

            {move || {
                status.get().map(|status| view! { <ReadOnly status=status age_of=age_of/> })
            }}

            <Show when=move || !replies.get().is_empty()>
                <h2 class="mt-2 text-sm font-medium opacity-70">"Replies"</h2>
                <For each=move || replies.get() key=|status| status.id.clone() let:status>
                    <ReadOnly status=status age_of=age_of/>
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
fn ReadOnly(status: Status, age_of: Callback<String, String>) -> impl IntoView {
    let age = age_of.run(status.created_at.clone());
    let account = status.account.clone();
    let name = display_name(&account.display_name, &account.acct);
    // A remote account's `acct` already carries its domain, and prefixing that with `@`
    // produces `@@` in the first character a reader sees.
    let handle = if account.acct.contains('@') {
        account.acct.clone()
    } else {
        format!("@{}", account.acct)
    };
    let avatar = account.avatar.clone();
    let alt = account.acct.clone();

    view! {
        <div class="card bg-base-100 shadow-sm">
            <div class="card-body gap-2 p-3">
                <header class="flex items-center gap-2">
                    <img
                        class="size-8 rounded-full bg-base-300 object-cover"
                        src=avatar
                        alt=alt
                        loading="lazy"
                        referrerpolicy="no-referrer"
                    />
                    <div class="min-w-0">
                        <p class="truncate text-sm font-semibold">{name}</p>
                        <p class="truncate text-xs opacity-60">{format!("{handle} · {age}")}</p>
                    </div>
                </header>
                {StatusBody(StatusBodyProps {
                    status,
                    on_open_media: None,
                    on_vote: None,
                    interactive: false,
                    revealed_init: false,
                })}
            </div>
        </div>
    }
}

/// Keeps a submit from reloading the page, on the target that has a page to reload.
fn stop_submit(event: &leptos::ev::SubmitEvent) {
    #[cfg(not(target_arch = "wasm32"))]
    let _ = event;
    #[cfg(target_arch = "wasm32")]
    event.prevent_default();
}
