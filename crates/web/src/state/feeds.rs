//! Which timelines the reader wants in the rail.
//!
//! A preference, not a permission: hiding "Local" does not stop the SDK from holding
//! the timeline, it stops the rail from offering a door to it. That is why this lives
//! in the browser rather than in the SDK's `AppState` — the SDK owns what the instance
//! said, this owns what the reader asked for.
//!
//! Stored as `nodus-feeds/1` with one `key=on|off` pair per line, so a build that grows
//! a fourth feed simply finds no entry for it and falls back to its default.

use core::cell::RefCell;

use leptos::prelude::*;

/// The feeds the rail can offer, in the order it offers them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Feed {
    Home,
    Local,
    Federated,
}

impl Feed {
    /// Every feed, in rail order.
    pub const ALL: [Self; 3] = [Self::Home, Self::Local, Self::Federated];

    /// The storage key and the query value.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::Home => "home",
            Self::Local => "local",
            Self::Federated => "federated",
        }
    }

    /// What the feed holds, for the settings row.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Home => "Home",
            Self::Local => "Local",
            Self::Federated => "Federated",
        }
    }

    /// Why a reader might turn it off.
    #[must_use]
    pub const fn description(self) -> &'static str {
        match self {
            Self::Home => "Posts from the accounts you follow.",
            Self::Local => {
                "Posts from accounts on your instance, whether you follow them or not."
            }
            Self::Federated => {
                "Posts your instance has received from other instances."
            }
        }
    }
}

/// `localStorage` key; the version lives in the key so a new record shape can pick a new
/// one instead of migrating.
const KEY: &str = "nodus-feeds/1";

thread_local! {
    /// Parsed preferences, filled on first read.
    static ENABLED: RefCell<Option<Vec<(Feed, bool)>>> = const { RefCell::new(None) };
    /// Bumped on every change so the rail re-renders.
    static REVISION: RefCell<Option<RwSignal<u64>>> = const { RefCell::new(None) };
}

/// Creates the change signal before the first mount.
///
/// Called from `main()` next to the other pre-mount installs: a signal created inside a
/// component belongs to it, and the rail reads this one after the reader leaves Settings.
pub fn install() {
    REVISION.with_borrow_mut(|slot| {
        if slot.is_none() {
            *slot = Some(RwSignal::new(0));
        }
    });
}

/// The signal a view reads to join this module's updates.
///
/// Returns `None` only before `install()`, which `main()` always runs.
#[must_use]
pub fn revision() -> Option<RwSignal<u64>> {
    REVISION.with_borrow(Clone::clone)
}

/// Whether `feed` is enabled; unknown feeds answer their default, which is on.
#[must_use]
pub fn enabled(feed: Feed) -> bool {
    with_prefs(|prefs| {
        prefs
            .iter()
            .find(|(known, _)| *known == feed)
            .is_none_or(|(_, on)| *on)
    })
}

/// Sets whether `feed` appears in the rail.
pub fn set(feed: Feed, on: bool) {
    with_prefs(|prefs| {
        prefs.retain(|(known, _)| *known != feed);
        prefs.push((feed, on));
    });
    save();
    REVISION.with_borrow(|slot| {
        if let Some(revision) = slot {
            let _ = revision.try_update(|generation| *generation += 1);
        }
    });
}

fn with_prefs<R>(f: impl FnOnce(&mut Vec<(Feed, bool)>) -> R) -> R {
    ENABLED.with_borrow_mut(|slot| {
        if slot.is_none() {
            *slot = Some(read().unwrap_or_default());
        }
        f(slot.as_mut().expect("just filled"))
    })
}

fn storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok()?
}

fn read() -> Option<Vec<(Feed, bool)>> {
    let raw = storage()?.get_item(KEY).ok()??;
    let prefs = raw
        .lines()
        .filter_map(|line| {
            let (key, value) = line.split_once('=')?;
            let feed = Feed::ALL.into_iter().find(|feed| feed.key() == key)?;
            Some((feed, value != "off"))
        })
        .collect();
    Some(prefs)
}

fn save() {
    let Some(storage) = storage() else {
        return;
    };
    let text = with_prefs(|prefs| {
        prefs
            .iter()
            .map(|(feed, on)| format!("{}={}", feed.key(), if *on { "on" } else { "off" }))
            .collect::<Vec<_>>()
            .join("\n")
    });
    let _ = storage.set_item(KEY, &text);
}
