//! Line icons.
//!
//! Drawn inline rather than loaded: the bundle already ships the whole app, and an icon
//! font or a sprite sheet would be one more request before the rail can paint. Each icon
//! is a path on a 24×24 grid with `currentColor`, so it inherits the text colour and can
//! sit inside any button or link without a wrapper.

use leptos::prelude::*;

/// The icons the app uses. Names are the meaning, not the drawing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    /// The recommendation feed.
    Sparkle,
    /// The home timeline.
    Home,
    /// Direct conversations.
    Mail,
    /// Notifications.
    Bell,
    /// Search.
    Search,
    /// People to follow.
    Users,
    /// Saved statuses.
    Bookmark,
    /// The scheduled queue.
    Clock,
    /// Hand-picked timelines.
    List,
    /// Writing a status.
    Pen,
    /// Settings.
    Settings,
    /// A boost.
    Repeat,
    /// A favourite.
    Star,
    /// A reply.
    Reply,
    /// A menu.
    More,
    /// Public.
    Globe,
    /// Followers only.
    Lock,
    /// Mentioned people only.
    At,
    /// A hashtag.
    Tag,
    /// Shown in the moderation screens.
    Shield,
    /// A warning or an error.
    Alert,
    /// A link out.
    Link,
    /// An image.
    Image,
    /// Something finished.
    Check,
    /// Something removed.
    Close,
    /// Something added.
    Plus,
}

impl Icon {
    /// The stroke paths, as SVG markup.
    pub const fn paths(self) -> &'static str {
        match self {
            Self::Sparkle => {
                r#"<path d="M12 3l1.9 4.6L18.5 9.5 13.9 11.4 12 16l-1.9-4.6L5.5 9.5l4.6-1.9z"/><path d="M18 15l.9 2.1L21 18l-2.1.9L18 21l-.9-2.1L15 18l2.1-.9z"/>"#
            }
            Self::Home => r#"<path d="M3 11l9-8 9 8"/><path d="M5 10v10h5v-6h4v6h5V10"/>"#,
            Self::Mail => {
                r#"<rect x="3" y="5" width="18" height="14" rx="2"/><path d="M3 7l9 6 9-6"/>"#
            }
            Self::Bell => {
                r#"<path d="M12 3a5 5 0 0 0-5 5v4l-2 4h14l-2-4V8a5 5 0 0 0-5-5z"/><path d="M10 20a2 2 0 0 0 4 0"/>"#
            }
            Self::Search => r#"<circle cx="11" cy="11" r="7"/><path d="M20 20l-3.5-3.5"/>"#,
            Self::Users => {
                r#"<circle cx="9" cy="8" r="3.5"/><path d="M2.5 20a6.5 6.5 0 0 1 13 0"/><path d="M16 5.5a3.5 3.5 0 0 1 0 6"/><path d="M17.5 14a6.5 6.5 0 0 1 4 6"/>"#
            }
            Self::Bookmark => r#"<path d="M6 3h12v18l-6-4.5L6 21z"/>"#,
            Self::Clock => r#"<circle cx="12" cy="12" r="9"/><path d="M12 7v5l3.5 2"/>"#,
            Self::List => {
                r#"<path d="M8 6h13"/><path d="M8 12h13"/><path d="M8 18h13"/><path d="M3 6h.01"/><path d="M3 12h.01"/><path d="M3 18h.01"/>"#
            }
            Self::Pen => {
                r#"<path d="M12 20h9"/><path d="M16.5 3.5a2.1 2.1 0 0 1 3 3L7 19l-4 1 1-4z"/>"#
            }
            Self::Settings => {
                r#"<circle cx="12" cy="12" r="3"/><path d="M4 12h2M18 12h2M12 4v2M12 18v2M6.3 6.3l1.4 1.4M16.3 16.3l1.4 1.4M17.7 6.3l-1.4 1.4M7.7 16.3l-1.4 1.4"/>"#
            }
            Self::Repeat => {
                r#"<path d="M17 3l4 4-4 4"/><path d="M3 11V9a2 2 0 0 1 2-2h16"/><path d="M7 21l-4-4 4-4"/><path d="M21 13v2a2 2 0 0 1-2 2H3"/>"#
            }
            Self::Star => {
                r#"<path d="M12 3.5l2.7 5.6 6.1.9-4.4 4.3 1 6.1-5.4-2.9-5.4 2.9 1-6.1L3.2 10l6.1-.9z"/>"#
            }
            Self::Reply => r#"<path d="M9 7l-5 5 5 5"/><path d="M4 12h10a6 6 0 0 1 6 6v1"/>"#,
            Self::More => {
                r#"<circle cx="5" cy="12" r="1.5"/><circle cx="12" cy="12" r="1.5"/><circle cx="19" cy="12" r="1.5"/>"#
            }
            Self::Globe => {
                r#"<circle cx="12" cy="12" r="9"/><path d="M3 12h18"/><path d="M12 3a14 14 0 0 1 0 18a14 14 0 0 1 0-18z"/>"#
            }
            Self::Lock => {
                r#"<rect x="4" y="10" width="16" height="11" rx="2"/><path d="M8 10V7a4 4 0 0 1 8 0v3"/>"#
            }
            Self::At => {
                r#"<circle cx="12" cy="12" r="4"/><path d="M16 8v5a3 3 0 0 0 6 0v-1a10 10 0 1 0-3.5 7.7"/>"#
            }
            Self::Tag => {
                r#"<path d="M3 12V5a2 2 0 0 1 2-2h7l9 9-9 9z"/><circle cx="8" cy="8" r="1.5"/>"#
            }
            Self::Shield => r#"<path d="M12 3l8 3v6c0 5-3.4 8.2-8 9-4.6-.8-8-4-8-9V6z"/>"#,
            Self::Alert => r#"<path d="M12 4l9 16H3z"/><path d="M12 10v4"/><path d="M12 17h.01"/>"#,
            Self::Link => {
                r#"<path d="M10 13a5 5 0 0 0 7 0l2-2a5 5 0 0 0-7-7l-1 1"/><path d="M14 11a5 5 0 0 0-7 0l-2 2a5 5 0 0 0 7 7l1-1"/>"#
            }
            Self::Image => {
                r#"<rect x="3" y="4" width="18" height="16" rx="2"/><circle cx="9" cy="9.5" r="1.5"/><path d="M4 18l5-5 4 4 3-3 4 4"/>"#
            }
            Self::Check => r#"<path d="M4 12.5l5 5L20 6.5"/>"#,
            Self::Close => r#"<path d="M6 6l12 12"/><path d="M18 6L6 18"/>"#,
            Self::Plus => r#"<path d="M12 5v14"/><path d="M5 12h14"/>"#,
        }
    }
}

/// One icon, sized by the enclosing text unless a class says otherwise.
#[component]
pub fn Icon(icon: Icon, #[prop(optional, into)] class: String) -> impl IntoView {
    let class = if class.is_empty() {
        "size-4 shrink-0".to_owned()
    } else {
        class
    };
    view! {
        <svg
            class=class
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="1.75"
            stroke-linecap="round"
            stroke-linejoin="round"
            aria-hidden="true"
            inner_html=icon.paths()
        ></svg>
    }
}
