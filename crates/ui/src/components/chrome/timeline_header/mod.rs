//! The heading above a timeline: what it is, why it exists, and where else to go.

use leptos::prelude::*;

use crate::TimelineTabs;

/// The heading of a timeline, its one sentence of orientation, and the tab strip when the
/// screen is one the strip can switch between.
///
/// `which` is the client's own name for the timeline — `home`, `feed`, `local`, `federated`,
/// `tag:<tag>` or `list:<id>` — so the heading is read off the name the caller already holds
/// and neither paint has to ask the router what page it is on. The sentence is a prop
/// because it belongs to the screen's copy, and a screen with nothing to say passes none:
/// the paragraph is then absent rather than empty.
///
/// The header markup is [`crate::PageHeader`]'s, written out here so that an absent sentence
/// removes the paragraph instead of leaving an empty one behind.
#[component]
#[expect(
    clippy::needless_pass_by_value,
    reason = "a Leptos component receives its props by value"
)]
pub fn TimelineHeader(
    /// The timeline's name, in the client's spelling.
    which: String,
    /// The sentence under the heading, when this screen has one.
    #[prop(optional)]
    description: Option<String>,
) -> impl IntoView {
    let title = title_of(&which);
    let tab = tab_for(&which);
    let description = description.filter(|sentence| !sentence.is_empty());
    view! {
        <div class="flex flex-col gap-2">
            <header class="flex flex-col gap-0.5">
                <h1 class="text-lg font-semibold">{title}</h1>
                {description
                    .map(|sentence| view! { <p class="text-sm opacity-60">{sentence}</p> })}
            </header>
            {tab
                .map(|tab| {
                    view! { <TimelineTabs active=Signal::derive(move || tab.to_owned())/> }
                })}
        </div>
    }
}

/// The heading for a timeline, read off the name the client keys it by.
///
/// A tag is written the way the app writes hashtags everywhere else. A list is known only by
/// its id, which says nothing a reader should see, so its word is all the heading can be. A
/// name this function does not recognise is shown as given: a wrong guess would be worse
/// than the caller's own spelling.
fn title_of(which: &str) -> String {
    if let Some(tag) = which.strip_prefix("tag:") {
        return format!("#{tag}");
    }
    if which.starts_with("list:") {
        return "List".to_owned();
    }
    match which {
        "home" => "Home".to_owned(),
        "feed" => "Feed".to_owned(),
        "local" => "Local".to_owned(),
        "federated" => "Federated".to_owned(),
        other => other.to_owned(),
    }
}

/// The tab strip's active address for a timeline, or `None` when the screen has no tab.
///
/// The strip draws its own four links, so this only answers which one to mark. Home and its
/// three siblings share one section, so the section stays lit on every one of them. A tag or
/// a list is reached from elsewhere.
fn tab_for(which: &str) -> Option<&'static str> {
    match which {
        "home" => Some("/home"),
        "feed" => Some("/home/feed"),
        "local" => Some("/home/local"),
        "federated" => Some("/home/federated"),
        _ => None,
    }
}
