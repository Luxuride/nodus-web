//! The author line of a status, shared by the server paint and the client mount.
//!
//! The line sits at the top of every card, so the two paints must agree on it to the pixel:
//! the avatar and the names reserve the same boxes on both, and the timestamp stays on the
//! right instead of dropping under the names when the client boots.
//!
//! Everything the line would otherwise look up — the profile address, the thread address,
//! the age in words — arrives as a prop. This crate reads no router and no clock, which is
//! what lets the server paint the same tree it hands to the browser.

use leptos::prelude::*;
use nodus_client_core::model::Status;

use crate::display_name;

/// Who wrote a status, and when.
///
/// The anchors carry real `href`s, so the server's paint is already a working link. On the
/// client `on_navigate` turns a press into a step inside the app; when it is absent the
/// browser's default is left alone.
#[component]
#[expect(
    clippy::needless_pass_by_value,
    reason = "a Leptos component receives its props by value"
)]
pub fn StatusHeader(
    /// The status this line introduces.
    status: Status,
    /// The publication time in words ("3h"), from [`crate::relative_age`].
    age: String,
    /// Where the author's profile lives.
    author_path: String,
    /// Where the post's own page lives.
    thread_path: String,
    /// Follows one of those addresses inside the app.
    #[prop(optional)]
    on_navigate: Option<Callback<String>>,
) -> impl IntoView {
    let name = display_name(&status.account.display_name, &status.account.acct);
    let avatar = status.account.avatar.clone();
    let handle = status.account.acct.clone();
    // The paths also live in stored values, because four click handlers read them: a
    // `String` captured by value would make the first handler `FnOnce`, and a stored value
    // is `Copy`.
    let author = StoredValue::new(author_path.clone());
    let thread = StoredValue::new(thread_path.clone());

    view! {
        <header class="flex items-center gap-3">
            <a
                class="avatar"
                href=author_path.clone()
                on:click=move |event| follow_click(&event, author, on_navigate)
            >
                <div class="w-10 rounded-full bg-base-300">
                    <img
                        width="40"
                        height="40"
                        decoding="async"
                        class="h-10 w-10 rounded-full object-cover"
                        src=avatar
                        alt=""
                        loading="lazy"
                        referrerpolicy="no-referrer"
                    />
                </div>
            </a>
            <div class="flex min-w-0 flex-1 flex-wrap items-baseline gap-x-1">
                <a
                    class="max-w-full truncate font-semibold leading-tight hover:underline"
                    href=author_path.clone()
                    on:click=move |event| follow_click(&event, author, on_navigate)
                >
                    {name}
                </a>
                <a
                    class="max-w-full truncate text-sm opacity-60 hover:underline"
                    href=author_path
                    on:click=move |event| follow_click(&event, author, on_navigate)
                >
                    {handle}
                </a>
            </div>
            <a
                class="text-xs whitespace-nowrap opacity-50 hover:underline"
                href=thread_path
                on:click=move |event| follow_click(&event, thread, on_navigate)
            >
                {age}
            </a>
        </header>
    }
}

/// Follows an anchor inside the app when there is a callback for it.
///
/// Without one the press is left alone: the `href` is the whole answer, which is what the
/// server's paint relies on (it emits no listener at all, and the client always passes a
/// callback here). The non-wasm build keeps a use of `event` so the listener argument is
/// not reported as unused on the target that has no `prevent_default`.
fn follow_click(
    event: &leptos::ev::MouseEvent,
    path: StoredValue<String>,
    on_navigate: Option<Callback<String>>,
) {
    #[cfg(not(target_arch = "wasm32"))]
    let _ = event;
    if let Some(on_navigate) = on_navigate {
        #[cfg(target_arch = "wasm32")]
        event.prevent_default();
        on_navigate.run(path.get_value());
    }
}
