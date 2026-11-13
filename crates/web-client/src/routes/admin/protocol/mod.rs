//! Protocol detection through the SDK, not a private probe.

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_pages::pages::admin::protocol::ProtocolPage;

use crate::state::alive::Alive;

#[component]
pub fn Protocol() -> impl IntoView {
    let (protocol, set_protocol) = signal(None::<String>);
    let (features, set_features) = signal(Vec::<String>::new());
    let (error, set_error) = signal(None::<String>);

    Effect::new(move |_| {
        let alive = Alive::new();
        spawn_local(async move {
            match nodus_client_sdk::protocol::detect("").await {
                Some(found) => {
                    let names = nodus_client_sdk::protocol::capabilities("").await;
                    alive.if_alive(|| {
                        set_protocol.set(Some(found.label().to_owned()));
                        set_features.set(names);
                        set_error.set(None);
                    });
                }
                None => alive.if_alive(|| {
                    set_error.set(Some("No supported client API answered.".to_owned()));
                }),
            }
        });
    });

    view! { <ProtocolPage protocol=protocol features=features error=error/> }
}
