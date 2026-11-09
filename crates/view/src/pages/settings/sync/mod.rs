//! The background-sync card's body and the confirmation both directions ask for.
//!
//! Two rules make the card honest, and both are visible in the markup. The badge never
//! prints "off" for a consent the server has not answered yet — an unknown consent is
//! unknown, and showing a default makes the page load look like a decision the reader just
//! made. And the button only asks; the caller's confirmation runs the write, because this
//! crate has no transport.
//!
//! The dialog is a `modal-open` div rather than a `<dialog>`, so it needs no imperative
//! `show_modal()` call and re-renders with the rest of the view.

use leptos::prelude::*;

/// What the operator is about to do, and what it costs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncIntent {
    /// Store the token and start syncing.
    Enable,
    /// Delete the stored token and stop.
    Disable,
}

impl SyncIntent {
    /// The heading of the confirmation.
    fn title(self) -> &'static str {
        match self {
            Self::Enable => "Start background sync?",
            Self::Disable => "Turn off background sync?",
        }
    }

    /// Exactly what the server will do, in the order it does it.
    fn consequences(self) -> &'static [&'static str] {
        match self {
            Self::Enable => &[
                "The server stores an encrypted copy of your access token.",
                "It reads your follows, favourites and boosts in the background, whether or not this app is open.",
                "It uses them to rank people for you; it never posts, follows or unfollows on your behalf.",
                "Interaction telemetry stays off — it is a separate consent.",
                "You can turn this off at any time, which deletes the stored token.",
            ],
            Self::Disable => &[
                "The server deletes its copy of your access token.",
                "Background syncing stops immediately; recommendations keep the last ranking they built.",
                "Your follows are then only read while this app is open.",
                "You can turn it back on later, which stores the token again.",
            ],
        }
    }

    /// The label of the button that confirms it.
    fn confirm_label(self) -> &'static str {
        match self {
            Self::Enable => "Start syncing",
            Self::Disable => "Delete my token",
        }
    }

    /// The button style: destroying data is not a primary action.
    fn confirm_class(self) -> &'static str {
        match self {
            Self::Enable => "btn btn-primary btn-sm",
            Self::Disable => "btn btn-error btn-sm",
        }
    }

    /// Whether this intent turns sync on.
    pub fn enables(self) -> bool {
        self == Self::Enable
    }
}

/// The consent's three states, its explanations, and the one button that changes it.
#[component]
pub fn SyncRows(
    /// What the server holds: `None` until it has answered.
    #[prop(into)]
    state: Signal<Option<bool>>,
    /// Whether a request is already on its way.
    #[prop(into)]
    busy: Signal<bool>,
    /// The last answer, when there was one.
    #[prop(into)]
    result: Signal<Option<String>>,
    /// Opens the confirmation for the opposite of what is held. With no callback the
    /// button stays disabled: a button that acts on a guess is worse than no button.
    #[prop(optional_no_strip)]
    on_ask: Option<Callback<()>>,
) -> impl IntoView {
    view! {
        <p class="text-sm opacity-80">
            "Turning this on lets the recommendation server keep an encrypted copy of your access token, sync your follows in the background, and rank without waiting for you to open the app. Turning it off deletes that copy. Interaction telemetry is a separate consent and stays off."
        </p>
        <p class="flex items-center gap-2 text-sm">
            <span
                class=move || match state.get() {
                    Some(true) => "badge badge-success badge-sm",
                    Some(false) => "badge badge-ghost badge-sm",
                    None => "badge badge-ghost badge-sm opacity-50",
                }
            >
                // Not "off" until the server says so: an unknown consent is unknown, and
                // showing a default makes the page load look like a decision the reader
                // just made.
                {move || match state.get() {
                    Some(true) => "on",
                    Some(false) => "off",
                    None => "…",
                }}
            </span>
            <span class="opacity-70">
                {move || match state.get() {
                    Some(true) => "This account is synced by the server.",
                    Some(false) => "Follows are only read while this app is open.",
                    None => "Asking the server what it holds…",
                }}
            </span>
        </p>
        <div class="card-actions justify-end">
            <button
                class=move || if state.get() == Some(true) {
                    "btn btn-ghost btn-sm"
                } else {
                    "btn btn-primary btn-sm"
                }
                disabled=move || state.get().is_none() || busy.get() || on_ask.is_none()
                on:click=move |_| {
                    if let Some(on_ask) = on_ask {
                        on_ask.run(());
                    }
                }
            >
                <Show when=move || busy.get()>
                    <span class="loading loading-spinner loading-xs"></span>
                </Show>
                {move || match state.get() {
                    Some(true) => "Turn off background sync",
                    Some(false) => "Enable background sync",
                    None => "Background sync",
                }}
            </button>
        </div>
        {move || result.get().map(|message| view! {
            <div class="alert alert-soft alert-info py-2 text-sm" role="status">
                <span>{message}</span>
            </div>
        })}
    }
}

/// The modal itself: a `modal-open` div rather than a `<dialog>`, so it needs no
/// imperative `show_modal()` call and re-renders with the rest of the view.
#[component]
pub fn ConfirmDialog(
    /// What is being confirmed, or `None` when nothing is.
    #[prop(into)]
    confirming: Signal<Option<SyncIntent>>,
    /// Closes the dialog without acting.
    #[prop(optional_no_strip)]
    on_cancel: Option<Callback<()>>,
    /// Runs the confirmed intent.
    #[prop(optional_no_strip)]
    on_confirm: Option<Callback<SyncIntent>>,
) -> impl IntoView {
    view! {
        <Show when=move || confirming.get().is_some()>
            <div class="modal modal-open" role="dialog" aria-modal="true">
                <div class="modal-box max-w-lg">
                    <h3 class="text-lg font-semibold">
                        {move || confirming.get().map_or("", SyncIntent::title)}
                    </h3>
                    <ul class="mt-3 flex list-disc flex-col gap-1 pl-5 text-sm opacity-80">
                        <For
                            each=move || {
                                confirming
                                    .get()
                                    .map_or(&[][..], SyncIntent::consequences)
                                    .to_vec()
                            }
                            key=|line| *line
                            let:line
                        >
                            <li>{line}</li>
                        </For>
                    </ul>
                    <div class="modal-action">
                        <button
                            class="btn btn-ghost btn-sm"
                            on:click=move |_| {
                                if let Some(on_cancel) = on_cancel {
                                    on_cancel.run(());
                                }
                            }
                        >
                            "Cancel"
                        </button>
                        <button
                            class=move || {
                                confirming.get().map_or("btn btn-primary btn-sm", SyncIntent::confirm_class)
                            }
                            on:click=move |_| {
                                if let (Some(intent), Some(on_confirm)) =
                                    (confirming.get_untracked(), on_confirm)
                                {
                                    on_confirm.run(intent);
                                }
                            }
                        >
                            {move || confirming.get().map_or("", SyncIntent::confirm_label)}
                        </button>
                    </div>
                </div>
            </div>
        </Show>
    }
}
