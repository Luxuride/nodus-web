//! The moment between the instance's answer and the app: "signing in".
//!
//! A server paints this after a callback too, so the card is shared and the *outcome* is a
//! prop: no error, one error with the sentence the instance or the transport gave, and a way
//! back to a fresh attempt. What is not here is the exchange itself — that is a network verb,
//! so it stays in the client and this card only reports where it stands.

use leptos::prelude::*;

use crate::components::panel::Panel;

/// The card shown while the callback is exchanged, and after it fails.
#[component]
pub fn SigningIn(
    /// The failure message, or `None` while the handshake is still in flight.
    #[prop(optional_no_strip)] error: Option<String>,
    /// Where "Back to sign-in" points. Absent when nobody can navigate (the server).
    #[prop(optional_no_strip)] back: Option<String>,
    #[prop(optional)] on_back: Option<Callback<()>>,
) -> impl IntoView {
    let error = StoredValue::new(error);
    let back = StoredValue::new(back);

    view! {
        <Panel title="Signing in">
            {move || match error.get_value() {
                None => view! {
                    <div class="flex items-center gap-2 text-sm opacity-70">
                        <span class="loading loading-spinner loading-xs"></span>
                        <span>"Finishing the handshake with your instance…"</span>
                    </div>
                }
                    .into_any(),
                Some(message) => view! {
                    <div class="flex flex-col gap-3">
                        <div class="alert alert-soft alert-error text-sm" role="alert">
                            <span>{message}</span>
                        </div>
                        <a
                            class="btn btn-ghost btn-sm self-start"
                            href=back.get_value().unwrap_or_else(|| "/".to_owned())
                            on:click=move |event| {
                                if let Some(on_back) = on_back {
                                    event.prevent_default();
                                    on_back.run(());
                                }
                            }
                        >
                            "Back to sign-in"
                        </a>
                    </div>
                }
                    .into_any(),
            }}
        </Panel>
    }
}
