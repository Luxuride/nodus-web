//! The one image viewer: a gallery over the whole app, closed by Escape or a click.
//!
//! One module and one mount point instead of a per-screen lightbox, because a second
//! implementation would drift (different close keys, different backdrop behaviour) and
//! the second screen would be the one nobody tested.

use std::cell::RefCell;

use leptos::prelude::*;
use wasm_bindgen::JsCast;

thread_local! {
    /// What is expanded, if anything. Created before the first mount by [`install`], so no
    /// component owns it and navigating away cannot dispose the viewer's only signal.
    static OPEN: RefCell<Option<RwSignal<Option<Gallery>>>> = const { RefCell::new(None) };
}

pub use nodus_view::media_grid::Picture;

/// Everything the viewer is looking at, and where in it they are.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Gallery {
    pictures: Vec<Picture>,
    index: usize,
}

impl Gallery {
    fn current(&self) -> Option<&Picture> {
        self.pictures.get(self.index)
    }

    fn len(&self) -> usize {
        self.pictures.len()
    }
}

/// Creates the viewer's signal and its key handler while no owner exists.
///
/// Call once, before mounting. The listener is on `document` because the reader expects
/// Escape to work without first clicking the picture — which is how a viewer that only
/// listens on its own element fails the one test people run.
pub fn install() {
    OPEN.with_borrow_mut(|slot| {
        if slot.is_none() {
            *slot = Some(RwSignal::new(None));
        }
    });

    let Some(document) = web_sys::window().and_then(|window| window.document()) else {
        return;
    };
    let handler = wasm_bindgen::closure::Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(
        move |event: web_sys::KeyboardEvent| {
            if open().get_untracked().is_none() {
                return;
            }
            match event.key().as_str() {
                "Escape" => close(),
                "ArrowLeft" => previous(),
                "ArrowRight" => next(),
                _ => return,
            }
            event.prevent_default();
        },
    );
    let _ = document.add_event_listener_with_callback("keydown", handler.as_ref().unchecked_ref());
    // Deliberately leaked: the viewer lives as long as the document, so the handler has
    // no lifetime it could be dropped at.
    handler.forget();
}

/// The viewer's signal.
///
/// # Panics
///
/// Panics if [`install`] has not run: a viewer whose signal lives in whichever component
/// happened to render first is exactly the disposed-signal bug this module avoids.
fn open() -> RwSignal<Option<Gallery>> {
    OPEN.with_borrow(|slot| slot.expect("media::install() must run before rendering"))
}

/// Opens the viewer on `pictures` at `index`, clamped into range.
pub fn expand(pictures: Vec<Picture>, index: usize) {
    if pictures.is_empty() {
        return;
    }
    let index = index.min(pictures.len() - 1);
    open().set(Some(Gallery { pictures, index }));
}

/// Opens the viewer on one picture.
pub fn expand_one(url: impl Into<String>, alt: impl Into<String>) {
    expand(
        vec![Picture {
            url: url.into(),
            alt: alt.into(),
        }],
        0,
    );
}

/// Closes the viewer, if it is open.
pub fn close() {
    open().set(None);
}

/// Moves to the next picture, wrapping so the last one leads back to the first.
pub fn next() {
    open().update(|slot| {
        if let Some(gallery) = slot {
            let len = gallery.len();
            if len > 0 {
                gallery.index = (gallery.index + 1) % len;
            }
        }
    });
}

/// Moves to the previous picture, wrapping.
pub fn previous() {
    open().update(|slot| {
        if let Some(gallery) = slot {
            let len = gallery.len();
            if len > 0 {
                gallery.index = (gallery.index + len - 1) % len;
            }
        }
    });
}

/// The viewer itself, mounted once by the shell.
#[component]
pub fn ImageModal() -> impl IntoView {
    let open = open();
    let gallery = move || open.get();
    let picture = move || gallery().as_ref().and_then(Gallery::current).cloned();
    let url = move || picture().map(|picture| picture.url).unwrap_or_default();
    let alt = move || picture().map(|picture| picture.alt).unwrap_or_default();
    let many = move || gallery().is_some_and(|gallery| gallery.len() > 1);
    let press = RwSignal::new(None::<(f64, f64)>);
    let swiped = RwSignal::new(false);
    let position = move || {
        gallery()
            .map(|gallery| format!("{} / {}", gallery.index + 1, gallery.len()))
            .unwrap_or_default()
    };

    let start = move |(x, y): (f64, f64)| {
        press.set(Some((x, y)));
        swiped.set(false);
    };
    let end = move |(x, y): (f64, f64)| {
        let Some((from_x, from_y)) = press.get_untracked() else {
            return;
        };
        press.set(None);
        let dx = x - from_x;
        let dy = y - from_y;
        // Horizontal, deliberate, and not a vertical scroll that wandered.
        if dx.abs() < 60.0 || dx.abs() <= dy.abs() {
            return;
        }
        swiped.set(true);
        if dx < 0.0 {
            next();
        } else {
            previous();
        }
    };

    view! {
        <nodus_view::Lightbox
            open=Signal::derive(move || gallery().is_some())
            url=Signal::derive(url)
            alt=Signal::derive(alt)
            many=Signal::derive(many)
            position=Signal::derive(position)
            swiped=Signal::derive(move || swiped.get())
            on_close=Some(Callback::new(|()| close()))
            on_previous=Some(Callback::new(|()| previous()))
            on_next=Some(Callback::new(|()| next()))
            on_press=Some(Callback::new(start))
            on_release=Some(Callback::new(end))
            on_cancel=Some(Callback::new(move |()| press.set(None)))
            on_swipe_consumed=Some(Callback::new(move |()| swiped.set(false)))
        />
    }
}
