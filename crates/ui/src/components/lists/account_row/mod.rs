//! One account in a list, shared by the server paint and the client mount.
//!
//! The people lists, the mute and block lists and the search results all show the same row:
//! the avatar, the name, the handle, and the one line the account wrote about itself. The
//! bio is HTML from the instance and arrives already sanitised, so it is injected with
//! `inner_html` under the same rule status bodies follow.
//!
//! The list-specific verb — unmute, unblock, follow — is not part of the row: which verb
//! applies depends on the list being read, so a screen that has one passes it in the `after`
//! slot. Without a slot the row renders nothing there at all.
//!
//! A missing avatar keeps its rounded box instead of collapsing, so a broken URL does not
//! move the name. A row with no bio has no bio line: an empty paragraph would only cost a
//! line box, and the box it would occupy is already the handle's.

use leptos::prelude::*;
use nodus_client_core::model::Account;

use crate::display_name;

/// One account: who they are and the line under their handle.
///
/// The anchors carry the profile address, so the server's paint is already a working link;
/// on the client `on_open` turns a press into a step inside the app and leaves the address
/// alone when the app has no callback for it.
#[component]
#[expect(
    clippy::needless_pass_by_value,
    reason = "a Leptos component receives its props by value"
)]
pub fn AccountRow(
    /// The account this row shows.
    account: Account,
    /// The instance's sanitised bio HTML, drawn under the handle.
    note: String,
    /// Opens the profile inside the app.
    #[prop(optional)]
    on_open: Option<Callback<String>>,
    /// The screen's own trailing control, such as an undo verb.
    #[prop(optional)]
    after: Option<ChildrenFn>,
) -> impl IntoView {
    let name = display_name(&account.display_name, &account.acct);
    let acct = account.acct.clone();
    let handle = format!("@{acct}");
    let avatar = account.avatar.clone();
    let has_avatar = !avatar.is_empty();
    let id = account.id.clone();
    let href = format!("/accounts/{id}");
    // The id is read by two handlers, so it lives in a stored value: a `String` captured by
    // value would make the first handler `FnOnce`, and a stored value is `Copy`.
    let target = StoredValue::new(id);

    view! {
        <li class="card border border-base-300 bg-base-100 shadow-sm">
            <div class="card-body flex-row items-start gap-3 p-3">
                <a
                    class="avatar"
                    href=href.clone()
                    on:click=move |event| open_account(&event, target, on_open)
                >
                    // The box is drawn whether or not there is a file: the client's row
                    // already reserves it, and a row that grows once the image lands moves
                    // everything under it.
                    <div class="w-10 rounded-full bg-base-300">
                        {has_avatar
                            .then(move || {
                                view! {
                                    <img
                                        class="h-10 w-10 rounded-full object-cover"
                                        width="40"
                                        height="40"
                                        decoding="async"
                                        loading="lazy"
                                        referrerpolicy="no-referrer"
                                        alt=""
                                        src=avatar
                                    />
                                }
                            })}
                    </div>
                </a>
                <div class="min-w-0 flex-1">
                    <a
                        class="font-medium hover:underline"
                        href=href
                        on:click=move |event| open_account(&event, target, on_open)
                    >
                        {name}
                    </a>
                    <p class="truncate text-xs opacity-60">{handle}</p>
                    {(!note.is_empty())
                        .then(move || {
                            view! {
                                <p class="line-clamp-2 text-sm opacity-70" inner_html=note></p>
                            }
                        })}
                </div>
                {after.map(|children| children())}
            </div>
        </li>
    }
}

/// Opens the profile inside the app when there is a callback for it.
///
/// Without one the press is left alone: the `href` is the whole answer, which is what the
/// server's paint relies on (it emits no listener at all). The non-wasm build keeps a use of
/// `event` so the listener argument is not reported as unused on the target that has no
/// `prevent_default`.
fn open_account(
    event: &leptos::ev::MouseEvent,
    id: StoredValue<String>,
    on_open: Option<Callback<String>>,
) {
    #[cfg(not(target_arch = "wasm32"))]
    let _ = event;
    if let Some(on_open) = on_open {
        #[cfg(target_arch = "wasm32")]
        event.prevent_default();
        on_open.run(id.get_value());
    }
}
