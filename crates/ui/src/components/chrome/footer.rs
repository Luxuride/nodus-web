//! The foot of every screen: what this is, and the two links a client owes its server.
//!
//! The instance address is a signal because the session that supplies it arrives after the
//! first paint. Without one the About link is absent from both paints, rather than pointing
//! at an empty host.

use leptos::prelude::*;

#[component]
pub fn AppFooter(#[prop(into)] host: Signal<Option<String>>) -> impl IntoView {
    view! {
        <footer class="mt-8 border-t border-base-300 px-4 py-6 text-xs opacity-70">
            <div class="mx-auto flex w-full max-w-5xl flex-wrap items-center gap-x-3 gap-y-2">
                <span class="font-semibold tracking-tight text-base-content">"nodus"</span>
                <span>"A Mastodon client with a recommendation feed."</span>
                {move || {
                    host.get()
                        .map(|host| {
                            view! {
                                <a
                                    class="link link-hover"
                                    href=format!("https://{host}/about")
                                    target="_blank"
                                    rel="noreferrer noopener"
                                >
                                    "About this instance"
                                </a>
                            }
                        })
                }}
                <a
                    class="link link-hover"
                    href="https://github.com/nodus-social"
                    target="_blank"
                    rel="noreferrer noopener"
                >
                    "Source"
                </a>
                <span class="ml-auto opacity-60">
                    "No ads, no tracking, no algorithm you cannot see."
                </span>
            </div>
        </footer>
    }
}
