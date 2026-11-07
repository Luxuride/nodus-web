//! The one image expander: a picture, at a size worth looking at, and a way out.
//!
//! One module and one mount point instead of a per-screen lightbox, because a second
//! implementation would drift (different close keys, different backdrop behaviour) and
//! the second screen would be the one nobody tested.

use std::cell::RefCell;

use leptos::prelude::*;

thread_local! {
    /// What is expanded, if anything. Created before the first mount by [`install`], so no
    /// component owns it and navigating away cannot dispose the modal's only signal.
    static OPEN: RefCell<Option<RwSignal<Option<Picture>>>> = const { RefCell::new(None) };
}

/// An image the operator asked to see larger.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Picture {
    /// The full-size image.
    pub url: String,
    /// Its text alternative, which doubles as the caption.
    pub alt: String,
}

/// Creates the open-picture signal while no owner exists. Call once, before mounting.
pub fn install() {
    OPEN.with_borrow_mut(|slot| {
        if slot.is_none() {
            *slot = Some(RwSignal::new(None));
        }
    });
}

/// The open-picture signal.
///
/// # Panics
///
/// Panics if [`install`] has not run: a modal whose signal lives in whichever component
/// happened to render first is exactly the disposed-signal bug this module avoids.
fn open() -> RwSignal<Option<Picture>> {
    OPEN.with_borrow(|slot| slot.expect("media::install() must run before rendering"))
}

/// Expands an image over the whole app.
pub fn expand(url: impl Into<String>, alt: impl Into<String>) {
    open().set(Some(Picture {
        url: url.into(),
        alt: alt.into(),
    }));
}

/// Closes the expanded image, if one is open.
pub fn close() {
    open().set(None);
}

/// The expander itself, mounted once by the shell.
#[component]
pub fn ImageModal() -> impl IntoView {
    let open = open();
    let current = move || open.get();
    let url = move || current().map(|picture| picture.url).unwrap_or_default();
    let alt = move || current().map(|picture| picture.alt).unwrap_or_default();

    view! {
        <Show when=move || current().is_some()>
            // A backdrop click closes, and so does every button in the tray: the two ways
            // out people try first, without a keybinding nobody discovers.
            <div class="modal modal-open" role="presentation" on:click=move |_| close()>
                <div
                    class="modal-box w-auto max-w-[95vw] bg-transparent p-0 shadow-none"
                    role="presentation"
                    on:click=|event| event.stop_propagation()
                >
                    <img
                        class="max-h-[85vh] max-w-[95vw] rounded-box bg-base-300 object-contain"
                        src=url
                        alt=alt
                        referrerpolicy="no-referrer"
                    />
                    <div class="mt-2 flex items-center justify-center gap-2">
                        <a
                            class="btn btn-sm"
                            href=url
                            target="_blank"
                            rel="nofollow noopener noreferrer"
                            referrerpolicy="no-referrer"
                        >
                            "Open original"
                        </a>
                        <button class="btn btn-sm btn-ghost" on:click=move |_| close()>
                            "Close"
                        </button>
                    </div>
                    <Show when=move || !alt().is_empty()>
                        <p class="mt-2 max-w-[95vw] text-center text-sm text-balance text-neutral-content/80">
                            {alt}
                        </p>
                    </Show>
                </div>
            </div>
        </Show>
    }
}
