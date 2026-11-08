//! One notification: who, what, and the status it is about when the instance inlined one.

use leptos::prelude::*;
use nodus_instance::model::{Notification, NotificationKind};

use super::super::format::relative_time;
use super::super::status::StatusCard;
use super::request::FollowRequestActions;

/// A single row, drawn by the crate the server draws with.
#[component]
pub fn NotificationRow(
    notification: Notification,
    reload: Callback<()>,
    set_error: WriteSignal<Option<String>>,
) -> impl IntoView {
    let age = relative_time(&notification.created_at);
    let is_request = notification.kind == NotificationKind::FollowRequest;
    let who = notification.account.id.clone();
    let status = notification.status.clone();

    let extra = is_request.then(|| {
        let who = who.clone();
        StoredValue::new(std::sync::Arc::new(move || {
            view! {
                <FollowRequestActions
                    account_id=who.clone()
                    on_done=reload
                    set_error=set_error
                />
            }
            .into_any()
        }) as ChildrenFn)
    });

    let body = status.map(|status| {
        StoredValue::new(std::sync::Arc::new(move || {
            view! {
                <ul class="flex flex-col gap-2">
                    <StatusCard status=status.clone()/>
                </ul>
            }
            .into_any()
        }) as ChildrenFn)
    });

    view! {
        <nodus_view::NotificationCard
            notification=notification
            age=age
            extra=extra.map(|extra| extra.get_value())
            body=body.map(|body| body.get_value())
        />
    }
}
