//! The offer to grant admin access, and the reason it is asked for at all.

use leptos::prelude::*;

/// Shown when no admin token is held.
///
/// The copy names the cost before the click: the consent screen the operator is about to
/// see asks for full moderation rights, and a button that hides that would be a trap.
#[component]
pub fn AdminGrant(
    /// Starts the authorization with admin scopes.
    granted: Callback<()>,
) -> impl IntoView {
    view! {
        <div class="flex flex-col gap-3">
            <p class="text-sm opacity-70">
                "Moderation lives behind a permission this app does not ask for at sign-in. Granting it authorizes a second token for this instance, with "
                <code class="text-xs">"admin:read"</code>
                " and "
                <code class="text-xs">"admin:write"</code>
                " — enough to read every report, suspend an account or block a domain."
            </p>
            <p class="text-sm opacity-70">
                "Your ordinary session keeps its own, narrower token; this grant is used only by the administration screens, and can be forgotten again at any time."
            </p>
            <button class="btn btn-sm w-fit" on:click=move |_| granted.run(())>
                "Grant admin access"
            </button>
        </div>
    }
}
