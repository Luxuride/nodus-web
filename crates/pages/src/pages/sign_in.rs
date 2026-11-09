//! The sign-in screen, painted by both sides.
//!
//! The instance field, the browser flow and the token escape hatch are markup; what the
//! caller owns is the address being typed, whether a request is in flight, and the words
//! that came back. The server paints the same form with every verb absent.

use leptos::prelude::*;

use crate::components::screen::{Screen, Width};

/// Sign in against an instance, with token pasting folded away.
#[component]
#[expect(
    clippy::too_many_lines,
    reason = "one form: every path has to be readable together"
)]
pub fn SignInPage(
    #[prop(optional_no_strip)] note: Option<String>,
    #[prop(into)] host: Signal<String>,
    #[prop(into)] token: Signal<String>,
    #[prop(into)] authorizing: Signal<bool>,
    #[prop(into)] pasting: Signal<bool>,
    #[prop(into)] authorize_error: Signal<Option<String>>,
    #[prop(into)] token_error: Signal<Option<String>>,
    #[prop(into)] startup_error: Signal<Option<String>>,
    #[prop(optional_no_strip)] on_host: Option<Callback<String>>,
    #[prop(optional_no_strip)] on_token: Option<Callback<String>>,
    #[prop(optional_no_strip)] on_authorize: Option<Callback<()>>,
    #[prop(optional_no_strip)] on_token_submit: Option<Callback<()>>,
) -> impl IntoView {
    let idle = move || !authorizing.get() && !pasting.get();

    view! {
        <Screen width=Width::Narrow>
            <div class="card border border-base-300 bg-base-100 shadow-sm">
                <div class="card-body gap-4">
                    <h1 class="card-title">"Sign in"</h1>
                    {note.map(|note| view! { <p class="text-sm font-medium">{note}</p> })}
                    <p class="text-sm opacity-70">
                        "Authorize this app on your instance. Approval happens on your instance's own page, and the access token it returns can be revoked there at any time."
                    </p>
                    <form
                        class="flex flex-col gap-3"
                        on:submit=move |event| {
                            event.prevent_default();
                            if let Some(on_authorize) = on_authorize {
                                on_authorize.run(());
                            }
                        }
                    >
                        <label class="form-control w-full">
                            <span class="label-text mb-1 text-sm">"Instance"</span>
                            <input
                                class="input input-bordered w-full"
                                type="text"
                                placeholder="mastodon.social"
                                autocomplete="url"
                                prop:value=move || host.get()
                                on:input=move |event| {
                                    if let Some(on_host) = on_host {
                                        on_host.run(event_target_value(&event));
                                    }
                                }
                                disabled=move || !idle()
                            />
                        </label>
                        <button class="btn btn-primary mt-2" type="submit" disabled=move || !idle()>
                            <Show when=move || authorizing.get()>
                                <span class="loading loading-spinner loading-xs"></span>
                            </Show>
                            "Authorize"
                        </button>
                    </form>
                    {move || {
                        authorize_error
                            .get()
                            .or_else(|| startup_error.get())
                            .map(|message| {
                                view! {
                                    <div class="alert alert-soft alert-error text-sm" role="alert">
                                        <span>{message}</span>
                                    </div>
                                }
                            })
                    }}
                    <details class="mt-2 border-t border-base-300 pt-3">
                        <summary class="cursor-pointer text-sm opacity-70">
                            "Advanced: sign in with an access token"
                        </summary>
                        <p class="mt-2 text-xs opacity-60">
                            "For an instance that does not support browser sign-in. Create a token in your instance's development settings with the read, write and follow scopes, then paste it here."
                        </p>
                        <form
                            class="mt-2 flex flex-col gap-3"
                            on:submit=move |event| {
                                event.prevent_default();
                                if let Some(on_token_submit) = on_token_submit {
                                    on_token_submit.run(());
                                }
                            }
                        >
                            <label class="form-control w-full">
                                <span class="label-text mb-1 text-sm">"Instance"</span>
                                <input
                                    class="input input-bordered input-sm w-full"
                                    type="text"
                                    placeholder="mastodon.social"
                                    autocomplete="url"
                                    prop:value=move || host.get()
                                    on:input=move |event| {
                                        if let Some(on_host) = on_host {
                                            on_host.run(event_target_value(&event));
                                        }
                                    }
                                    disabled=move || !idle()
                                />
                            </label>
                            <label class="form-control w-full">
                                <span class="label-text mb-1 text-sm">"Access token"</span>
                                <input
                                    class="input input-bordered w-full"
                                    type="password"
                                    autocomplete="off"
                                    prop:value=move || token.get()
                                    on:input=move |event| {
                                        if let Some(on_token) = on_token {
                                            on_token.run(event_target_value(&event));
                                        }
                                    }
                                    disabled=move || !idle()
                                />
                            </label>
                            <button
                                class="btn btn-secondary mt-1"
                                type="submit"
                                disabled=move || !idle()
                            >
                                <Show when=move || pasting.get()>
                                    <span class="loading loading-spinner loading-xs"></span>
                                </Show>
                                "Sign in with token"
                            </button>
                        </form>
                        {move || {
                            token_error
                                .get()
                                .map(|message| {
                                    view! {
                                        <div class="alert alert-soft alert-error mt-2 text-sm" role="alert">
                                            <span>{message}</span>
                                        </div>
                                    }
                                })
                        }}
                    </details>
                </div>
            </div>
        </Screen>
    }
}
