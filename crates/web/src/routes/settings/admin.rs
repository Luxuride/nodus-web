//! Administration, from the settings screen: who may join, and who is waiting.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::router::Route;
use crate::state::alive::Alive;
use crate::state::bridge::{admin, admin_host, forget_admin_token};
use crate::state::oauth;

mod grant;

use grant::AdminGrant;

/// Registration control, plus the way into the moderation screens.
///
/// Three answers, not two: "no grant yet" is not the same as "the instance said no", and
/// only the first one can be fixed from here. A boolean would have shown a moderator the
/// sentence meant for a stranger, which is exactly what happened while the app requested
/// no admin scopes at all.
#[component]
pub fn AdministrationCard() -> impl IntoView {
    let (verdict, set_verdict) = signal(None::<admin::Staff>);
    let alive = Alive::new();

    Effect::new(move |_| {
        spawn_local(async move {
            let answer = admin::staff().await;
            alive.if_alive(|| set_verdict.set(Some(answer)));
        });
    });

    let granted = Callback::new(move |()| {
        spawn_local(async move {
            if let Some(host) = admin_host() {
                let _ = oauth::begin_admin(&host).await;
                // On success the browser is already leaving for the instance.
            }
        });
    });

    let forget = Callback::new(move |()| {
        forget_admin_token();
        spawn_local(async move {
            let answer = admin::staff().await;
            alive.if_alive(|| set_verdict.set(Some(answer)));
        });
    });

    view! {
        <Show when=move || verdict.get().is_none()>
            <p class="text-sm opacity-60">"Checking what this account may do…"</p>
        </Show>

        <Show when=move || verdict.get() == Some(admin::Staff::NeedsGrant)>
            <AdminGrant granted=granted/>
        </Show>

        <Show when=move || verdict.get() == Some(admin::Staff::No)>
            <div class="flex flex-col gap-2">
                <p class="text-sm opacity-70">
                    "This account cannot moderate on this instance. Moderation rights are granted by the instance, not by Nodus."
                </p>
                <button class="btn btn-ghost btn-xs w-fit" on:click=move |_| forget.run(())>
                    "Forget the admin grant"
                </button>
            </div>
        </Show>

        <Show when=move || verdict.get() == Some(admin::Staff::Yes)>
            <div class="flex flex-col gap-3">
                <div class="card border border-base-300 bg-base-100 shadow-sm">
                    <div class="card-body gap-2 p-4">
                        <h2 class="card-title text-base">"Administration"</h2>
                        <p class="text-sm opacity-70">
                            "Reports, account standing, registrations, federation and
                             announcements each have their own section under /admin."
                        </p>
                        <div class="card-actions mt-1 flex-wrap gap-2">
                            <a
                                class="btn btn-primary btn-sm"
                                href=Route::Admin(crate::router::ADMIN_DEFAULT.to_owned()).path()
                                on:click=move |event| crate::router::link(
                                    event,
                                    Route::Admin(crate::router::ADMIN_DEFAULT.to_owned()),
                                )
                            >
                                "Open administration"
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

                <button class="btn btn-ghost btn-xs w-fit" on:click=move |_| forget.run(())>
                    "Forget the admin grant"
                </button>
            </div>
        </Show>
    }
}
