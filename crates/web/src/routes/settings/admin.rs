//! Administration, from the settings screen: who may join, and who is waiting.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::router::Route;
use crate::state::alive::Alive;
use crate::state::bridge::admin;

mod approvals;
mod invites;

use approvals::Approvals;
use invites::Invites;

/// Registration control, plus the way into the moderation screens.
///
/// Gated by a probe rather than by a role field: an account knows what it can do when the
/// server answers, and a card that leads to a 403 is worse than one that explains itself.
#[component]
pub fn AdministrationCard() -> impl IntoView {
    let (allowed, set_allowed) = signal(None::<bool>);
    let alive = Alive::new();

    Effect::new(move |_| {
        spawn_local(async move {
            let verdict = admin::is_moderator().await;
            alive.if_alive(|| set_allowed.set(Some(verdict)));
        });
    });

    view! {
        <Show when=move || allowed.get().is_none()>
            <p class="text-sm opacity-60">"Checking what this account may do…"</p>
        </Show>

        <Show when=move || allowed.get() == Some(false)>
            <p class="text-sm opacity-70">
                "Registration control and the moderation queue belong to moderators of this instance, and this account is not one."
            </p>
        </Show>

        <Show when=move || allowed.get() == Some(true)>
            <div class="flex flex-col gap-3">
                <Approvals/>
                <Invites/>

                <div class="card border border-base-300 bg-base-100 shadow-sm">
                    <div class="card-body gap-2 p-4">
                        <h2 class="card-title text-base">"Moderation"</h2>
                        <p class="text-sm opacity-70">
                            "Reports, account standing, federation and announcements live in their own screen."
                        </p>
                        <div class="card-actions mt-1 gap-2">
                            <a
                                class="btn btn-ghost btn-sm"
                                href=Route::Admin.path()
                                on:click=move |event| crate::router::link(event, Route::Admin)
                            >
                                "Open moderation"
                            </a>
                            <a
                                class="btn btn-ghost btn-sm"
                                href=Route::Mutes.path()
                                on:click=move |event| crate::router::link(event, Route::Mutes)
                            >
                                "Muted accounts"
                            </a>
                            <a
                                class="btn btn-ghost btn-sm"
                                href=Route::Blocks.path()
                                on:click=move |event| crate::router::link(event, Route::Blocks)
                            >
                                "Blocked accounts"
                            </a>
                        </div>
                    </div>
                </div>
            </div>
        </Show>
    }
}
