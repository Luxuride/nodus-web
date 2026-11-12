//! One status and its conversation.

use std::sync::Arc;

use leptos::prelude::*;
use nodus_client_core::model::{Context, Status};

use crate::components::CardRenderer;
use crate::pages::reply_card::ReplyCard;
use crate::relative_age;

use super::super::{Action, ActionResult, Runtime};

#[component]
#[expect(
    clippy::too_many_lines,
    reason = "the anonymous and signed-in reads share one body"
)]
pub fn ThreadScreen(
    runtime: Arc<dyn Runtime>,
    host: String,
    id: String,
    card: CardRenderer,
) -> AnyView {
    let local = host.trim().is_empty() && runtime.session().is_some();
    if host.trim().is_empty() && !local {
        let typed = RwSignal::new(String::new());
        let target = id.clone();
        return view! {
            <crate::pages::status::public::HostForm
                typed=Signal::derive(move || typed.get())
                on_input=Callback::new(move |value: String| typed.set(value))
                on_read=Callback::new(move |()| {
                    let host = typed.get_untracked().trim().to_owned();
                    if host.is_empty() {
                        return;
                    }
                    runtime.navigate(format!("/status/{host}/{target}"));
                })
            />
        }
        .into_any();
    }

    let status = RwSignal::new(runtime.status(&id));
    let ancestors = RwSignal::new(
        runtime
            .context(&id)
            .map(|context| context.ancestors)
            .unwrap_or_default(),
    );
    let replies = RwSignal::new(
        runtime
            .context(&id)
            .map(|context| context.descendants)
            .unwrap_or_default(),
    );
    let ready = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);
    let alive = crate::Alive::new();

    let load_id = id.clone();
    let load_host = host.clone();
    let load_status = {
        let runtime = Arc::clone(&runtime);
        let browse_host = host.clone();
        let browse_id = id.clone();
        Callback::new(move |()| {
            let runtime = Arc::clone(&runtime);
            let browse_runtime = Arc::clone(&runtime);
            let browse_host = browse_host.clone();
            let browse_id = browse_id.clone();
            let status_done = Callback::new(move |result: Result<Status, String>| {
                alive.if_alive(|| match result {
                    Ok(found) => {
                        if !browse_host.trim().is_empty() {
                            browse_runtime.note_browse(
                                "status",
                                browse_host.clone(),
                                browse_id.clone(),
                                Some(found.account.acct.clone()),
                                found.url.clone(),
                            );
                        }
                        status.set(Some(found));
                    }
                    Err(message) => error.set(Some(message)),
                });
            });
            let context_done = Callback::new(move |result: Result<Context, String>| {
                alive.if_alive(|| match result {
                    Ok(context) => {
                        ancestors.set(context.ancestors);
                        replies.set(context.descendants);
                    }
                    Err(message) => error.set(Some(message)),
                });
                ready.set(true);
            });
            if load_host.is_empty() {
                runtime.load_thread(load_id.clone(), status_done, context_done);
            } else {
                runtime.load_public_thread(
                    load_host.clone(),
                    load_id.clone(),
                    status_done,
                    context_done,
                );
            }
        })
    };
    let prefetched = runtime.status(&id).is_some() && runtime.context(&id).is_some();
    if prefetched {
        ready.set(true);
    } else {
        Effect::new(move |_| load_status.run(()));
    }

    let age_of = Callback::new({
        let runtime = Arc::clone(&runtime);
        move |created: String| relative_age(&created, runtime.now_millis())
    });
    let reply_slot = runtime.session().is_some().then(|| {
        runtime.reply_composer(id.clone(), replies).unwrap_or_else(|| {
            Arc::new({
                let runtime = Arc::clone(&runtime);
                let parent = id.clone();
                move || {
                    view! {
                        <ReplyComposer runtime=Arc::clone(&runtime) parent=parent.clone() replies=replies />
                    }
                    .into_any()
                }
            })
        })
    });
    let card = StoredValue::new(card);
    let reply_slot: StoredValue<Option<ChildrenFn>> = StoredValue::new(reply_slot);
    let age_of = StoredValue::new(age_of);

    view! {
        <Show
            when=move || status.get().is_some() && ready.get()
            fallback=move || {
                view! {
                    <crate::Screen>
                        <Show when=move || error.get().is_some()>
                            <crate::Note class="text-error text-xs">
                                {move || error.get().unwrap_or_default()}
                            </crate::Note>
                        </Show>
                    </crate::Screen>
                }
                .into_any()
            }
        >
            {move || {
                let card = card.get_value();
                let reply_slot = reply_slot.get_value();
                let age_of = age_of.get_value();
                status.get().map(|focused| {
                    view! {
                        <crate::thread::ThreadPage
                            ancestors=ancestors.get()
                            focused=focused
                            replies=replies
                            card=card.clone()
                            age_of=age_of
                            reply_slot=reply_slot.clone()
                        />
                    }
                })
            }}
        </Show>
    }
    .into_any()
}

#[component]
fn ReplyComposer(
    runtime: Arc<dyn Runtime>,
    parent: String,
    replies: RwSignal<Vec<Status>>,
) -> impl IntoView {
    let text = RwSignal::new(String::new());
    let busy = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);

    let submit = Callback::new(move |()| {
        let draft = text.get_untracked().trim().to_owned();
        if draft.is_empty() {
            return;
        }
        busy.set(true);
        error.set(None);
        let done = Callback::new(move |result: Result<ActionResult, String>| {
            busy.set(false);
            match result {
                Ok(ActionResult::Status(reply)) => {
                    text.set(String::new());
                    replies.update(|replies| replies.push(reply));
                }
                Ok(_) => {}
                Err(message) => error.set(Some(message)),
            }
        });
        runtime.action(
            Action::Reply {
                parent: parent.clone(),
                text: draft,
            },
            done,
        );
    });

    view! {
        <ReplyCard
            text=text.get()
            busy=busy.get()
            on_input=Callback::new(move |draft: String| text.set(draft))
            on_submit=submit
        />
        <Show when=move || error.get().is_some()>
            <crate::Note class="text-error text-xs">
                {move || error.get().unwrap_or_default()}
            </crate::Note>
        </Show>
    }
}
