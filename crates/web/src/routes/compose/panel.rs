//! The composer as a panel rather than a page.
//!
//! Writing a post is a short interruption, not a destination: leaving the timeline to
//! do it loses the reply context the user was reading and costs a navigation each way.
//! The panel keeps the composer beside the timeline on a wide screen, above the rail
//! when there is no room for a third column, and full width on a phone.
//!
//! The signals are created before the first mount ([`install`] runs from `main`), for
//! the same reason every other app-lifetime signal is: one created inside a component is
//! disposed with it, and the second Compose press would read a disposed handle.

use leptos::prelude::*;

use super::form::ComposeForm;

thread_local! {
    static PANEL: Panel = Panel::new();
}

#[derive(Clone, Copy)]
struct Panel {
    open: RwSignal<bool>,
}

impl Panel {
    fn new() -> Self {
        Self {
            open: RwSignal::new(false),
        }
    }
}

/// Create the panel's state. Called from `main` before the first mount.
pub fn install() {
    PANEL.with(|_| ());
}

fn panel() -> Panel {
    PANEL.with(|panel| *panel)
}

/// Reactive open flag, for the layout that gives the panel its column.
#[must_use]
pub fn open() -> RwSignal<bool> {
    panel().open
}

/// Open the panel, or close it when it is already showing.
pub fn toggle() {
    let panel = panel();
    let next = !panel.open.get_untracked();
    panel.open.set(next);
}

/// Close the panel after a successful post.
pub fn close() {
    panel().open.set(false);
}

/// The panel itself: a card around [`ComposeForm`], with its own close control.
#[component]
pub fn ComposerPanel() -> impl IntoView {
    let close = move |_: web_sys::MouseEvent| self::close();
    // Bound to a local: a turbofish cannot appear inside a `view!` attribute.
    let initial: Option<nodus_instance::model::Status> = None;
    view! {
        <section class="card bg-base-100 shadow-sm">
            <div class="card-body gap-3 p-4">
                <div class="flex items-center justify-between">
                    <h2 class="card-title text-base">"New post"</h2>
                    <button class="btn btn-ghost btn-xs" on:click=close>
                        "Close"
                    </button>
                </div>
                <ComposeForm initial=initial/>
            </div>
        </section>
    }
}
