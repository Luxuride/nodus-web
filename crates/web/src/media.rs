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

/// One image the reader asked to see larger.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Picture {
    /// The full-size image.
    pub url: String,
    /// Its text alternative, which doubles as the caption.
    pub alt: String,
}

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

    view! {
        <Show when=move || gallery().is_some()>
            // The backdrop is the close button: no tray, no labels, nothing to read. Escape
            // does the same thing for the reader with a keyboard.
            <div
                class="modal modal-open cursor-zoom-out"
                role="presentation"
                on:click=move |_| close()
            >
                <Frame url=Signal::derive(url) alt=Signal::derive(alt) press=press swiped=swiped/>

                <Show when=many>
                    // Arrow keys work too; these exist because a phone has no arrows.
                    <button
                        type="button"
                        class="btn btn-circle btn-sm absolute top-1/2 left-2 -translate-y-1/2 cursor-pointer"
                        aria-label="Previous image"
                        on:click=move |event| {
                            event.stop_propagation();
                            previous();
                        }
                    >
                        "‹"
                    </button>
                    <button
                        type="button"
                        class="btn btn-circle btn-sm absolute top-1/2 right-2 -translate-y-1/2 cursor-pointer"
                        aria-label="Next image"
                        on:click=move |event| {
                            event.stop_propagation();
                            next();
                        }
                    >
                        "›"
                    </button>
                    <p class="absolute bottom-3 left-1/2 -translate-x-1/2 rounded-box bg-base-300/70 px-2 py-0.5 text-xs">
                        {position}
                    </p>
                </Show>

                <Show when=move || !alt().is_empty()>
                    <p class="absolute top-3 left-1/2 max-w-[90vw] -translate-x-1/2 rounded-box bg-base-300/70 px-2 py-1 text-center text-xs text-balance">
                        {alt}
                    </p>
                </Show>
            </div>
        </Show>
    }
}

/// The image itself, and the gesture that moves through the gallery.
///
/// A separate component because the drag state is three closures and the viewer already
/// had more lines than the lint allows.
#[component]
fn Frame(
    url: Signal<String>,
    alt: Signal<String>,
    press: RwSignal<Option<(f64, f64)>>,
    swiped: RwSignal<bool>,
) -> impl IntoView {
    let start = move |event: &web_sys::PointerEvent| {
        press.set(Some((
            f64::from(event.client_x()),
            f64::from(event.client_y()),
        )));
        swiped.set(false);
    };
    let end = move |event: &web_sys::PointerEvent| {
        let Some((from_x, from_y)) = press.get_untracked() else {
            return;
        };
        press.set(None);
        let dx = f64::from(event.client_x()) - from_x;
        let dy = f64::from(event.client_y()) - from_y;
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
        <div
            class="pointer-events-none absolute inset-0 flex items-center justify-center p-4"
            role="presentation"
        >
            <img
                class="pointer-events-auto max-h-[92vh] max-w-[96vw] touch-pan-y rounded-box object-contain"
                src=move || url.get()
                alt=move || alt.get()
                referrerpolicy="no-referrer"
                on:pointerdown=move |event| {
                    event.stop_propagation();
                    start(&event);
                }
                on:pointerup=move |event| {
                    event.stop_propagation();
                    end(&event);
                }
                on:pointercancel=move |_| press.set(None)
                on:click=move |event| {
                    // A swipe that ends on the image must not read as "close".
                    if swiped.get_untracked() {
                        swiped.set(false);
                        event.stop_propagation();
                    }
                }
            />
        </div>
    }
}
