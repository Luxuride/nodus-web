//! Line icons, shared by both paints.
//!
//! Drawn inline rather than loaded: the bundle already ships the whole app, and an icon
//! font or a sprite sheet would be one more request before the rail can paint. Each icon
//! is a path on a 24×24 grid with `currentColor`, so it inherits the text colour and can
//! sit inside any button or link without a wrapper.

use leptos::prelude::*;

/// The icons the app uses. Names are the meaning, not the drawing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(
    dead_code,
    reason = "the icon set is one list; a build uses the part its screens need"
)]
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
    /// The glyph, as SVG markup from the Lucide set.
    ///
    /// The drawing is not this crate's to invent: Lucide is the set Mastodon's own clients use
    /// and its geometry is drawn on the same 24×24 grid with the same stroke, so a glyph here
    /// looks like the glyph in a screenshot of the web client. Only the *meaning* is ours —
    /// `Icon::Sparkle` is "the recommendation feed", whatever the picture is called upstream.
    pub fn glyph(self) -> icondata::Icon {
        match self {
            Self::Sparkle => icondata::LuSparkles,
            Self::Home => icondata::LuHome,
            Self::Repeat => icondata::LuRepeat2,
            Self::Star => icondata::LuStar,
            Self::Reply => icondata::LuReply,
            Self::More => icondata::LuMoreHorizontal,
            Self::Globe => icondata::LuGlobe,
            Self::Lock => icondata::LuLock,
            Self::At => icondata::LuAtSign,
            Self::Tag => icondata::LuTag,
            Self::Shield => icondata::LuShield,
            Self::Alert => icondata::LuAlertTriangle,
            Self::Link => icondata::LuLink,
            Self::Image => icondata::LuImage,
            Self::Check => icondata::LuCheck,
            Self::Close => icondata::LuX,
            Self::Plus => icondata::LuPlus,
            Self::Mail => icondata::LuMail,
            Self::Bell => icondata::LuBell,
            Self::Search => icondata::LuSearch,
            Self::Users => icondata::LuUsers,
            Self::Bookmark => icondata::LuBookmark,
            Self::Clock => icondata::LuClock,
            Self::List => icondata::LuList,
            Self::Pen => icondata::LuPenLine,
            Self::Settings => icondata::LuSettings,
        }
    }

    /// The glyph's SVG body.
    pub fn paths(self) -> &'static str {
        self.glyph().data
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
    let glyph = icon.glyph();
    view! {
        <svg
            class=class
            viewBox=glyph.view_box.unwrap_or("0 0 24 24")
            fill=glyph.fill.unwrap_or("none")
            stroke=glyph.stroke.unwrap_or("currentColor")
            stroke-width=glyph.stroke_width.unwrap_or("2")
            stroke-linecap=glyph.stroke_linecap.unwrap_or("round")
            stroke-linejoin=glyph.stroke_linejoin.unwrap_or("round")
            aria-hidden="true"
            inner_html=glyph.data
        ></svg>
    }
}
