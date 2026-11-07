//! One notification: who, what, and the status it is about when the instance inlined one.

use leptos::prelude::*;
use nodus_instance::model::{Notification, NotificationKind};

use super::super::format::{display_name, relative_time};
use super::super::status::StatusCard;
use super::request::FollowRequestActions;

/// What kind of attention this was.
fn describe(kind: NotificationKind) -> &'static str {
    match kind {
        NotificationKind::Follow => "followed you",
        NotificationKind::FollowRequest => "asked to follow you",
        NotificationKind::Reblog => "boosted your post",
        NotificationKind::Favourite => "favourited your post",
        NotificationKind::Mention => "mentioned you",
        NotificationKind::Poll => "A poll you voted in has ended",
        NotificationKind::Status => "posted",
        NotificationKind::AdminSignUp => "A new account signed up",
        NotificationKind::AdminReport => "A report needs review",
    }
}

/// A single row.
#[component]
#[expect(
    clippy::needless_pass_by_value,
    reason = "a Leptos component receives its props by value"
)]
pub fn NotificationRow(
    notification: Notification,
    reload: Callback<()>,
    set_error: WriteSignal<Option<String>>,
) -> impl IntoView {
    let who = notification.account.clone();
    let heading = display_name(&who.display_name, &who.acct);
    let when = relative_time(&notification.created_at);
    let said = describe(notification.kind);
    let profile = format!("/accounts/{}", who.id);
    let status = notification.status.clone();
    let is_request = notification.kind == NotificationKind::FollowRequest;

    view! {
        <li class="card border border-base-300 bg-base-100 shadow-sm">
            <div class="card-body gap-2 p-4">
                <div class="flex items-center gap-2">
                    <a class="avatar" href=profile.clone()>
                        <div class="w-8 rounded-full bg-base-300">
                            <img src=who.avatar.clone() alt="" referrerpolicy="no-referrer"/>
                        </div>
                    </a>
                    <p class="min-w-0 flex-1 truncate text-sm">
                        <a class="font-semibold hover:underline" href=profile>{heading}</a>
                        <span class="opacity-70">{format!(" {said}")}</span>
                    </p>
                    <time class="text-xs opacity-50" datetime=notification.created_at.clone()>
                        {when}
                    </time>
                </div>

                <Show when=move || is_request>
                    <FollowRequestActions
                        account_id=who.id.clone()
                        on_done=reload
                        set_error=set_error
                    />
                </Show>

                {status
                    .map(|status| {
                        view! {
                            <ul class="flex flex-col gap-2">
                                <StatusCard status=status/>
                            </ul>
                        }
                    })}
            </div>
        </li>
    }
}
