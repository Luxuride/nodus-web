//! The column every page is drawn in, and the three widths a page may choose.

use leptos::prelude::*;
/// How wide a screen's content column is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Width {
    /// Timelines and settings: the reading column.
    Standard,
    /// Moderation and administration: tables of decisions.
    Wide,
    /// Sign-in and the OAuth callback: one card, nothing beside it.
    Narrow,
}

impl Width {
    #[must_use]
    pub const fn class(self) -> &'static str {
        match self {
            Self::Standard => "max-w-2xl",
            Self::Wide => "max-w-3xl",
            Self::Narrow => "max-w-md",
        }
    }
}

/// A screen's content column.
///
/// One wrapper for every screen on both paints. The padding belongs to the frame, not to
/// the screen, so a page that adds its own `px-4` is a page whose column is four units
/// narrower than the same page painted by the server — which is the width jump this
/// component exists to remove.
#[component]
pub fn Screen(
    #[prop(default = Width::Standard)] width: Width,
    children: Children,
) -> impl IntoView {
    view! {
        <section class=format!("mx-auto flex w-full flex-col gap-3 px-2 {}", width.class())>
            {children()}
        </section>
    }
}
