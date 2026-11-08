//! Who this browser is signed in as, and the way out.

use leptos::prelude::*;

use crate::state::bridge::sign_out;

/// The current session, plus the one control that always applies to it.
#[component]
pub(crate) fn SessionCard(
    host: impl Fn() -> String + Copy + Send + Sync + 'static,
    acct: impl Fn() -> String + Copy + Send + Sync + 'static,
) -> impl IntoView {
    view! {
        <div class="card mb-4 border border-base-300 bg-base-100 shadow-sm">
            <div class="card-body gap-2 p-4">
                <h2 class="card-title text-base">"Session"</h2>
                <dl class="grid grid-cols-[8rem_minmax(0,1fr)] gap-x-3 gap-y-1 text-sm">
                    <dt class="opacity-60">"Instance"</dt>
                    <dd class="truncate">{move || host()}</dd>
                    <dt class="opacity-60">"Account"</dt>
                    <dd class="truncate">{move || acct()}</dd>
                </dl>
                <div class="card-actions mt-2 justify-end">
                    <button class="btn btn-ghost btn-sm" on:click=move |_| sign_out()>
                        "Sign out"
                    </button>
                </div>
                <p class="text-xs opacity-60">
                    "Signing out forgets the token stored in this browser."
                </p>
            </div>
        </div>
    }
}
