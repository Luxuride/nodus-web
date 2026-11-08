//! The emoticon button and the picker that hangs under it.
//!
//! Kept out of [`super::bar`] for two reasons: the bar is already at the edge of clippy's
//! `too_many_lines`, and the button owns a document listener, which is the kind of thing a
//! reader should be able to find on its own.

use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;

type ClickListener = Closure<dyn FnMut(web_sys::MouseEvent)>;

thread_local! {
    /// The document listener that puts the picker away. One composer can have its picker
    /// open at a time (the panel is a popover over the bar), so one slot is enough — and a
    /// `Closure` is neither `Send` nor `Sync`, which rules out keeping it in a signal.
    static LISTENER: std::cell::RefCell<Option<ClickListener>> = const { std::cell::RefCell::new(None) };
}

/// Opens the emoji picker, and closes it when the writer presses anywhere else.
#[component]
pub(super) fn EmojiToggle(
    /// Whether the picker is showing.
    open: ReadSignal<bool>,
    /// Toggles it.
    set_open: WriteSignal<bool>,
    /// Disables the button while a post is in flight.
    busy: ReadSignal<bool>,
) -> impl IntoView {
    let host = NodeRef::<leptos::html::Div>::new();

    Effect::new(move |_| {
        let showing = open.get();
        let listening = LISTENER.with(|slot| slot.borrow().is_some());
        if showing && !listening {
            let listener = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(
                move |event: web_sys::MouseEvent| {
                    let Some(target) = event
                        .target()
                        .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
                    else {
                        return;
                    };
                    let inside = host.get().is_some_and(|node| {
                        let host: web_sys::Element = node.unchecked_into();
                        host.contains(Some(&target))
                    });
                    let toggle = target
                        .closest("[data-emoji-toggle]")
                        .ok()
                        .flatten()
                        .is_some();
                    if !inside && !toggle {
                        set_open.set(false);
                    }
                },
            );
            if let Some(window) = web_sys::window() {
                let _ = window
                    .add_event_listener_with_callback("click", listener.as_ref().unchecked_ref());
            }
            LISTENER.with(|slot| *slot.borrow_mut() = Some(listener));
        }
        if !showing {
            LISTENER.with(|slot| {
                if let Some(listener) = slot.borrow_mut().take() {
                    if let Some(window) = web_sys::window() {
                        let _ = window.remove_event_listener_with_callback(
                            "click",
                            listener.as_ref().unchecked_ref(),
                        );
                    }
                }
            });
        }
    });

    view! {
        // The emoticon sits with the counter, not with the compose-your-post row: it edits
        // the body, and the left group is about what kind of post this is.
        <div class="relative" node_ref=host>
            <button
                type="button"
                data-emoji-toggle
                class="btn btn-outline btn-sm px-2 text-lg leading-none"
                class:bg-base-200=move || open.get()
                title="Insert an emoji"
                aria-expanded=move || open.get().to_string()
                disabled=move || busy.get()
                on:click=move |_| set_open.update(|showing| *showing = !*showing)
            >
                <span class="text-xl leading-none">"☺"</span>
            </button>
            // Absolutely positioned so opening the picker does not push the bar taller and
            // move the button the writer just pressed.
            <crate::ui::editor::EmojiPanel open=open/>
        </div>
    }
}
