//! One notification: who did it, what they did, and the status it happened to.

use leptos::prelude::*;

use super::format::{describe_notification, display_name};
/// One notification: who, what, when, and whatever the caller renders beneath.
///
/// `body` is the status the instance inlined (a card on both paints) and `extra` is the
/// client's follow-request controls, which the server has no business drawing. Everything
/// else — the avatar, the phrase, the age, the card — is here, so the two paints cannot
/// drift apart on it.
#[component]
#[expect(
    clippy::needless_pass_by_value,
    reason = "a Leptos component receives its props by value"
)]
pub fn NotificationCard(
    #[prop(into)] notification: nodus_instance::model::Notification,
    #[prop(into)] age: String,
    body: Option<ChildrenFn>,
    extra: Option<ChildrenFn>,
) -> impl IntoView {
    let who = notification.account.clone();
    let heading = display_name(&who.display_name, &who.acct);
    let said = describe_notification(notification.kind);
    let profile = format!("/accounts/{}", who.id);
    let created_at = notification.created_at.clone();
    let avatar = who.avatar.clone();
    let body = StoredValue::new(body);
    let extra = StoredValue::new(extra);

    view! {
        <li class="card border border-base-300 bg-base-100 shadow-sm">
            <div class="card-body gap-2 p-4">
                <div class="flex items-center gap-2">
                    <a class="avatar" href=profile.clone()>
                        <div class="w-8 rounded-full bg-base-300">
                            <img
                                width="40"
                                height="40"
                                decoding="async"
                                src=avatar
                                alt=""
                                referrerpolicy="no-referrer"
                            />
                        </div>
                    </a>
                    <p class="min-w-0 flex-1 truncate text-sm">
                        <a class="font-semibold hover:underline" href=profile>
                            {heading}
                        </a>
                        <span class="opacity-70">{format!(" {said}")}</span>
                    </p>
                    <time class="text-xs opacity-50" datetime=created_at>{age}</time>
                </div>
                {extra.with_value(|extra| extra.as_ref().map(|extra| extra()))}
                <div class="nodus-notification-body min-w-0">
                    {body.with_value(|body| body.as_ref().map(|body| body()))}
                </div>
            </div>
        </li>
    }
}
