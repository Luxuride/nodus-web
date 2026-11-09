//! The shell a phone gets: a bar at the bottom, a sheet for everything else.
//!
//! The rail is the right shape for a wide window — twelve destinations, always in the same
//! place, each one visible. On a phone the same list eats a third of the screen before the
//! first post, so the narrow layout keeps the five destinations a phone is used to having
//! under the thumb and puts the rest behind one button. Nothing becomes unreachable: the
//! sheet is the rail, in a drawer.
//!
//! The destinations arrive as data (`BarTab`) because this crate has no `Route`: the client
//! builds the list from its router, the server never renders a phone shell. Every row is an
//! `<a href>`; when the caller supplies `on_navigate` the press is intercepted (that is what
//! makes it a client-side change instead of a document load), and when it does not — as in the
//! server — the href is the whole behaviour.

use leptos::prelude::*;

use crate::components::icons::Icon;
use crate::components::rail::icon_of;

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
    let home_path = StoredValue::new(if home_path.is_empty() { "/".to_owned() } else { home_path });
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
    /// Whether the sheet is open; the "More" cell toggles it.
    open: RwSignal<bool>,
    /// The path the app is on, so one cell can be marked current.
    current: Signal<String>,
    /// The cells, in reading order.
    tabs: Vec<BarTab>,
    #[prop(optional_no_strip)] on_navigate: Option<Callback<String>>,
) -> impl IntoView {
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
                    <button
                        type="button"
                        class="flex w-full flex-col items-center gap-0.5 py-2 text-[0.65rem] font-medium leading-none"
                        on:click=move |_| open.update(|shown| *shown = !*shown)
                        aria-label="All destinations"
                    >
                        <Icon icon=Icon::More class="size-5"/>
                        <span class="max-w-full truncate">"More"</span>
                    </button>
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
                class="flex flex-col items-center gap-0.5 py-2 text-[0.65rem] font-medium leading-none"
                class:opacity-60=move || current.get() != path.get_value()
                class:text-primary=move || current.get() == path.get_value()
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

/// The rest of the destinations, in a sheet that slides up from the bottom bar.
///
/// A sheet rather than a page: opening it must not change the address bar, so closing it
/// returns the reader to exactly the screen they were on.
#[component]
pub fn MobileSheet(
    /// Whether the sheet is showing.
    open: RwSignal<bool>,
    /// The path the app is on, so one row can be marked current.
    current: Signal<String>,
    /// The groups, in reading order.
    groups: Vec<SheetGroup>,
    #[prop(optional_no_strip)] on_navigate: Option<Callback<String>>,
) -> impl IntoView {
    view! {
        <div
            class="fixed inset-0 z-50 xl:hidden"
            class:hidden=move || !open.get()
            role="dialog"
            aria-modal="true"
            aria-label="All destinations"
        >
            <button
                type="button"
                class="absolute inset-0 w-full bg-black/40"
                aria-label="Close"
                on:click=move |_| open.set(false)
            ></button>
            <div class="absolute inset-x-0 bottom-0 max-h-[80vh] overflow-y-auto rounded-t-2xl border-t border-base-300 bg-base-100 pb-24 shadow-2xl">
                <div class="flex items-center justify-between px-4 pt-3">
                    <h2 class="text-sm font-semibold">"Everything else"</h2>
                    <button
                        type="button"
                        class="btn btn-ghost btn-sm btn-square"
                        aria-label="Close"
                        on:click=move |_| open.set(false)
                    >
                        <Icon icon=Icon::Close/>
                    </button>
                </div>
                <ul class="menu w-full px-2 py-2">
                    <For each=move || groups.clone() key=|group| group.title.clone() let:group>
                        <li class="menu-title">{group.title.clone()}</li>
                        <For each=move || group.items.clone() key=|item| item.path.clone() let:item>
                            <SheetRow
                                tab=item
                                current=current
                                open=open
                                on_navigate=on_navigate
                            />
                        </For>
                    </For>
                </ul>
            </div>
        </div>
    }
}

/// One row in the sheet: closes the sheet, then navigates, so the reader sees the screen
/// rather than the drawer sliding away over it.
#[component]
fn SheetRow(
    tab: BarTab,
    current: Signal<String>,
    open: RwSignal<bool>,
    #[prop(optional_no_strip)] on_navigate: Option<Callback<String>>,
) -> impl IntoView {
    let path = StoredValue::new(tab.path.clone());
    let label = tab.label;
    view! {
        <li>
            <a
                class="flex w-full items-center gap-2 py-2 font-medium"
                class:menu-active=move || current.get() == path.get_value()
                href=path.get_value()
                on:click=move |event| {
                    open.set(false);
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
