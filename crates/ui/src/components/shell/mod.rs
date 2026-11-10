//! The shell a phone gets: a bar at the bottom, and one page for everything else.
//!
//! The rail is the right shape for a wide window — twelve destinations, always in the same
//! place, each one visible. On a phone the same list eats a third of the screen before the
//! first post, so the narrow layout keeps the five destinations a phone is used to having
//! under the thumb and puts the rest behind one destination. Nothing becomes unreachable:
//! More is the rail, as a page of its own, so the bar stays under the thumb while it is open.
//!
//! The destinations arrive as data (`BarTab`) because this crate has no `Route`: the client
//! builds the list from its router, the server never renders a phone shell. Every row is an
//! `<a href>`; when the caller supplies `on_navigate` the press is intercepted (that is what
//! makes it a client-side change instead of a document load), and when it does not — as in the
//! server — the href is the whole behaviour.

#![expect(
    clippy::needless_pass_by_value,
    reason = "Leptos components receive their props by value"
)]

use leptos::prelude::*;

use crate::components::icons::Icon;
use crate::components::page_header::PageHeader;
use crate::components::rail::{icon_of, within};
use crate::components::screen::Screen;

/// One destination: a label, a picture, and where it goes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BarTab {
    /// The word under the icon, and the accessible name.
    pub label: String,
    /// The rail's picture for it.
    pub icon: Icon,
    /// An in-app path, e.g. `/notifications`.
    pub path: String,
}

impl BarTab {
    /// Builds one row.
    pub fn new(label: impl Into<String>, icon: Icon, path: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            icon,
            path: path.into(),
        }
    }

    /// Builds a row whose picture is the icon for its label, like the rail does.
    #[must_use]
    pub fn of(label: impl Into<String>, path: impl Into<String>) -> Self {
        let label = label.into();
        Self {
            icon: icon_of(&label),
            label,
            path: path.into(),
        }
    }
}

/// A titled group of rows in the sheet ("Read", "Find", "Library", "Account").
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SheetGroup {
    /// The group's heading, drawn as a list title.
    pub title: String,
    /// The destinations under it.
    pub items: Vec<BarTab>,
}

/// The bar above the bottom bar's content: who this is, and the one action worth a
/// permanent button.
#[component]
pub fn MobileTop(
    /// Where the wordmark points, e.g. `/`.
    #[prop(optional, into)]
    home_path: String,
    #[prop(optional)] on_home: Option<Callback<()>>,
) -> impl IntoView {
    let home_path = StoredValue::new(if home_path.is_empty() {
        "/".to_owned()
    } else {
        home_path
    });
    view! {
        <header class="sticky top-0 z-30 border-b border-base-300 bg-base-200/95 backdrop-blur xl:hidden">
            <div class="mx-auto flex max-w-2xl items-center gap-2 px-3 py-2">
                <a
                    class="font-semibold tracking-tight"
                    href=home_path.get_value()
                    on:click=move |event| {
                        if let Some(on_home) = on_home {
                            event.prevent_default();
                            on_home.run(());
                        }
                    }
                >
                    "nodus"
                </a>
                // No "Write" button: the composer already sits above the home timeline,
                // one line below this bar, and a second door to it duplicated a target
                // the reader is about to use anyway.
            </div>
        </header>
    }
}

/// The five destinations a phone gets under the thumb, in reading order.
#[component]
pub fn MobileBar(
    /// The path the app is on, so one cell can be marked current.
    current: Signal<String>,
    /// The cells, in reading order.
    tabs: Vec<BarTab>,
    /// The cell for everything else; its path is the More page's address.
    more: BarTab,
    /// Whether a destination in More is the one being shown.
    #[prop(into)]
    more_active: Signal<bool>,
    #[prop(optional_no_strip)] on_navigate: Option<Callback<String>>,
) -> impl IntoView {
    let more_path = StoredValue::new(more.path.clone());
    let more_label = more.label.clone();
    let more_icon = more.icon;
    view! {
        <nav
            class="fixed inset-x-0 bottom-0 z-40 border-t border-base-300 bg-base-100/95 backdrop-blur xl:hidden"
            style="padding-bottom: env(safe-area-inset-bottom)"
            aria-label="Primary"
        >
            <ul class="mx-auto flex max-w-lg items-stretch px-1">
                <For each=move || tabs.clone() key=|tab| tab.path.clone() let:tab>
                    <BarCell tab=tab current=current on_navigate=on_navigate/>
                </For>
                <li class="flex-1">
                    <a
                        class="flex min-h-12 flex-col items-center justify-center gap-0.5 py-2.5 text-[0.7rem] font-medium leading-none"
                        class:opacity-60=move || !more_active.get()
                        class:text-primary=move || more_active.get()
                        aria-current=move || more_active.get().then_some("page")
                        href=more_path.get_value()
                        title=more_label.clone()
                        on:click=move |event| {
                            if let Some(on_navigate) = on_navigate {
                                event.prevent_default();
                                on_navigate.run(more_path.get_value());
                            }
                        }
                    >
                        <Icon icon=more_icon class="size-5"/>
                        <span class="max-w-full truncate">{more_label.clone()}</span>
                    </a>
                </li>
            </ul>
        </nav>
    }
}

/// One destination in the bottom bar: icon above the label, and the whole cell is the hit
/// area. The label is small because five of them have to fit on a 320 px screen, and it is
/// truncated rather than wrapped because a two-line bar would change height with the route.
#[component]
fn BarCell(
    tab: BarTab,
    current: Signal<String>,
    #[prop(optional_no_strip)] on_navigate: Option<Callback<String>>,
) -> impl IntoView {
    let path = StoredValue::new(tab.path.clone());
    let label = tab.label;
    view! {
        <li class="flex-1">
            <a
                class="flex min-h-12 flex-col items-center justify-center gap-0.5 py-2.5 text-[0.7rem] font-medium leading-none"
                class:opacity-60=move || !within(&path.get_value(), &current.get())
                class:text-primary=move || within(&path.get_value(), &current.get())
                aria-current=move || within(&path.get_value(), &current.get()).then_some("page")
                href=path.get_value()
                on:click=move |event| {
                    if let Some(on_navigate) = on_navigate {
                        event.prevent_default();
                        on_navigate.run(path.get_value());
                    }
                }
                title=label.clone()
            >
                <Icon icon=tab.icon class="size-5"/>
                <span class="max-w-full truncate">{label.clone()}</span>
            </a>
        </li>
    }
}

/// Every destination the bottom bar does not hold, as a page that keeps the bar visible.
///
/// It is a route, not a drawer: the address bar says `/more`, the back button closes it, and
/// the bar underneath still marks the section the reader came from.
#[component]
pub fn MorePage(
    /// The path the app is on, so one row can be marked current.
    current: Signal<String>,
    /// The groups, in reading order.
    groups: Vec<SheetGroup>,
    #[prop(optional_no_strip)] on_navigate: Option<Callback<String>>,
) -> impl IntoView {
    view! {
        <Screen>
            <PageHeader
                title="More"
                description="Everything the bottom bar keeps out of the way."
            />
            <ul class="menu w-full gap-1 rounded-box bg-base-100 p-2 shadow-sm">
                <For each=move || groups.clone() key=|group| group.title.clone() let:group>
                    <li class="menu-title">{group.title.clone()}</li>
                    <For each=move || group.items.clone() key=|item| item.path.clone() let:item>
                        <MoreRow tab=item current=current on_navigate=on_navigate/>
                    </For>
                </For>
            </ul>
        </Screen>
    }
}

/// One row in More: the whole row is a link, and its section stays marked inside a submenu.
#[component]
fn MoreRow(
    tab: BarTab,
    current: Signal<String>,
    #[prop(optional_no_strip)] on_navigate: Option<Callback<String>>,
) -> impl IntoView {
    let path = StoredValue::new(tab.path.clone());
    let label = tab.label;
    view! {
        <li>
            <a
                class="flex w-full items-center gap-2 py-2 font-medium"
                class:menu-active=move || within(&path.get_value(), &current.get())
                aria-current=move || within(&path.get_value(), &current.get()).then_some("page")
                href=path.get_value()
                on:click=move |event| {
                    if let Some(on_navigate) = on_navigate {
                        event.prevent_default();
                        on_navigate.run(path.get_value());
                    }
                }
            >
                <Icon icon=tab.icon/>
                <span>{label}</span>
            </a>
        </li>
    }
}
