//! The strip of sections a staff area opens with.
//!
//! Moderation and administration each carried a private copy of this markup, and both copies
//! wrote the anchor, the icon and the active class by hand. One component taking the sections
//! as data keeps the two strips identical; `on_navigate` is the only difference between the
//! paints — with it a press is a router step, without it each tab is a plain link.

use leptos::prelude::*;

use crate::icons::Icon;

/// One section of a staff area: where it goes, what it says, and how it is drawn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SectionTab {
    /// The last path segment, appended to the strip's base.
    pub slug: String,
    /// The label the reader sees.
    pub label: String,
    /// The sentence a hover shows.
    pub hint: String,
    /// The drawing beside the label.
    pub icon: Icon,
}

/// The section strip, one link each.
///
/// `active` is the slug of the section the address names, so a section counts as selected by
/// its own name rather than by a whole path either paint would have to spell.
#[component]
pub fn SectionStrip(
    /// The path every section hangs off, without a trailing slash: `/moderation`, `/admin`.
    #[prop(into)]
    base: String,
    /// The sections, in the order they are shown.
    tabs: Vec<SectionTab>,
    /// The slug of the section on screen.
    #[prop(into)]
    active: String,
    /// Opens a section inside the app. Absent on the server, where the anchor is enough.
    ///
    /// `optional_no_strip` so an area page can forward its own optional verb unchanged: a
    /// stripped setter takes a bare `Callback`, and the anchor must keep its own navigation
    /// when there is none.
    #[prop(optional_no_strip)]
    on_navigate: Option<Callback<String>>,
) -> impl IntoView {
    view! {
        <div role="tablist" class="tabs tabs-box flex-wrap">
            <For each=move || tabs.clone() key=|tab| tab.slug.clone() let:tab>
                {
                    let here = active.clone();
                    let is_active = tab.slug == here;
                    let href = format!("{base}/{}", tab.slug);
                    let slug = tab.slug.clone();
                    view! {
                        <a
                            role="tab"
                            class="tab gap-2"
                            class:tab-active=is_active
                            href=href
                            title=tab.hint.clone()
                            on:click=move |event| {
                                if let Some(on_navigate) = on_navigate {
                                    event.prevent_default();
                                    on_navigate.run(slug.clone());
                                }
                            }
                        >
                            <Icon icon=tab.icon class="size-4"/>
                            <span>{tab.label.clone()}</span>
                        </a>
                    }
                }
            </For>
        </div>
    }
}
