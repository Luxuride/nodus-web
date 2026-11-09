//! What a reader sees before signing in.

use leptos::prelude::*;

/// The signed-out page's body.
#[component]
pub fn LandingPage() -> impl IntoView {
    view! {
        <div class="mx-auto flex w-full flex-col gap-3 px-2 max-w-2xl">
            <h1 class="text-xl font-semibold tracking-tight">"Nodus"</h1>
            <p class="text-sm opacity-70">
                "A Mastodon client that keeps its state, so a page you have already read does not load again."
            </p>
            <p class="text-sm opacity-70">"Sign in to see your timelines."</p>
        </div>
    }
}
