//! The confirmation both directions of the sync toggle ask for.
//!
//! Enabling hands the server a token and starts it processing an account's data;
//! disabling deletes what it holds. Neither is a toggle-sized decision, so the button
//! opens this instead of acting.

use leptos::prelude::*;

/// What the operator is about to do, and what it costs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Intent {
    /// Store the token and start syncing.
    Enable,
    /// Delete the stored token and stop.
    Disable,
}

impl Intent {
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
    pub(super) fn enables(self) -> bool {
        self == Self::Enable
    }
}

/// The modal itself: a `modal-open` div rather than a `<dialog>`, so it needs no
/// imperative `show_modal()` call and re-renders with the rest of the view.
#[component]
pub(super) fn ConfirmDialog(
    confirming: RwSignal<Option<Intent>>,
    run: Callback<Intent>,
) -> impl IntoView {
    view! {
        <Show when=move || confirming.get().is_some()>
                <div class="modal modal-open" role="dialog" aria-modal="true">
                    <div class="modal-box max-w-lg">
                        <h3 class="text-lg font-semibold">
                            {move || confirming.get().map_or("", Intent::title)}
                        </h3>
                        <ul class="mt-3 flex list-disc flex-col gap-1 pl-5 text-sm opacity-80">
                            <For
                                each=move || {
                                    confirming
                                        .get()
                                        .map_or(&[][..], |intent| intent.consequences())
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
                                on:click=move |_| confirming.set(None)
                            >
                                "Cancel"
                            </button>
                            <button
                                class=move || {
                                    confirming.get().map_or("btn btn-primary btn-sm", Intent::confirm_class)
                                }
                                on:click=move |_| {
                                    if let Some(intent) = confirming.get_untracked() {
                                        run.run(intent);
                                    }
                                }
                            >
                                {move || confirming.get().map_or("", Intent::confirm_label)}
                            </button>
                        </div>
                    </div>
                </div>
        </Show>
    }
}
