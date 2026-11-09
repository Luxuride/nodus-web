//! The three answers a staff probe can give, and what each one paints.
//!
//! Moderation and administration ask the same question — may this account see the panels —
//! and both answered it in the same three-armed shape: still asking, a grant to make, or a
//! refusal. The markup lived in the client, so the server could not paint it at all; it is
//! one component now, and the section behind the gate arrives as its children, which keeps
//! the three arms in one place whichever screen is behind them.

use leptos::prelude::*;

/// What a staff probe answered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StaffStanding {
    /// The token opened the report queue: the panels work.
    Yes,
    /// The instance refused: this account is not a moderator, whatever scopes it holds.
    No,
    /// No admin token was granted, so nothing can be said yet.
    NeedsGrant,
}

/// The three answers, and the section that is only drawn for `Yes`.
///
/// `standing` is `None` while the probe is out. `on_grant` asks the instance for the wider
/// token and retries the probe; where it is absent the button keeps its box and is disabled,
/// which is what a paint with no session to grant against shows.
#[component]
pub fn StaffGate(
    /// What the probe answered, or `None` while it is out.
    standing: Signal<Option<StaffStanding>>,
    /// Asks for `admin:read` and `admin:write`, then retries. Absent where there is nowhere
    /// to ask.
    ///
    /// `optional_no_strip`, not `optional`: the page above forwards whatever its own caller
    /// gave it, which may be `None`, and a stripped setter would take a bare `Callback` and
    /// turn `None` into a present (therefore enabled) button.
    #[prop(optional_no_strip)]
    on_grant: Option<Callback<()>>,
    /// The section the gate opens on `Yes`.
    children: ChildrenFn,
) -> impl IntoView {
    // Taken out of the view once, so the gate can be read as three arms rather than one
    // nested `Show`.
    let body = StoredValue::new(children);

    view! {
        <Show when=move || standing.get().is_none()>
            <p class="text-sm opacity-70">"Checking what this account may do…"</p>
        </Show>

        <Show when=move || matches!(standing.get(), Some(StaffStanding::NeedsGrant))>
            <div class="card border border-base-300 bg-base-100 shadow-sm">
                <div class="card-body gap-3 p-4">
                    <h2 class="card-title text-base">"This needs a moderation token"</h2>
                    <p class="text-sm opacity-70">
                        "Moderation lives behind a permission this app does not ask for at
                         sign-in. Granting it authorizes a second token for this instance,
                         with "
                        <code class="text-xs">"admin:read"</code>
                        " and "
                        <code class="text-xs">"admin:write"</code>
                        " — enough to read every report, suspend an account or block a domain."
                    </p>
                    <p class="text-sm opacity-70">
                        "Your ordinary session keeps its own, narrower token; this grant is
                         used only here, and can be forgotten again at any time."
                    </p>
                    <button
                        class="btn btn-sm w-fit"
                        disabled=on_grant.is_none()
                        on:click=move |_| {
                            if let Some(on_grant) = on_grant {
                                on_grant.run(());
                            }
                        }
                    >
                        "Grant admin access"
                    </button>
                </div>
            </div>
        </Show>

        <Show when=move || matches!(standing.get(), Some(StaffStanding::No))>
            <div class="card border border-base-300 bg-base-100 shadow-sm">
                <div class="card-body gap-2 p-4">
                    <h2 class="card-title text-base">"Not a moderator here"</h2>
                    <p class="text-sm opacity-70">
                        "This account has no staff role on this instance, so nothing here
                         would work. An admin has to give it one; that is not something a
                         client can do."
                    </p>
                </div>
            </div>
        </Show>

        <Show when=move || matches!(standing.get(), Some(StaffStanding::Yes))>
            {body.with_value(|body| body())}
        </Show>
    }
}
