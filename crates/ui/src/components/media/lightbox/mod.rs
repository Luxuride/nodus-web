//! The image viewer's chrome: the backdrop, the arrows, the caption and the count.
//!
//! Which picture is open and how the gallery moves is the caller's state; this module only
//! paints it. The gesture arithmetic stays with the caller too, because it needs the same
//! press/release bookkeeping the keyboard handler uses.

use leptos::prelude::*;

/// One picture, filling the screen.
#[component]
pub fn Lightbox(
    #[prop(into)] open: Signal<bool>,
    #[prop(into)] url: Signal<String>,
    #[prop(into)] alt: Signal<String>,
    #[prop(into)] many: Signal<bool>,
    #[prop(into)] position: Signal<String>,
    #[prop(into)] swiped: Signal<bool>,
    #[prop(optional_no_strip)] on_close: Option<Callback<()>>,
    #[prop(optional_no_strip)] on_previous: Option<Callback<()>>,
    #[prop(optional_no_strip)] on_next: Option<Callback<()>>,
    #[prop(optional_no_strip)] on_press: Option<Callback<(f64, f64)>>,
    #[prop(optional_no_strip)] on_release: Option<Callback<(f64, f64)>>,
    #[prop(optional_no_strip)] on_cancel: Option<Callback<()>>,
    #[prop(optional_no_strip)] on_swipe_consumed: Option<Callback<()>>,
) -> impl IntoView {
    view! {
        <Show when=move || open.get()>
            <div
                class="modal modal-open cursor-zoom-out"
                role="presentation"
                on:click=move |_| {
                    if let Some(on_close) = on_close {
                        on_close.run(());
                    }
                }
            >
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
                            if let Some(on_press) = on_press {
                                on_press.run((
                                    f64::from(event.client_x()),
                                    f64::from(event.client_y()),
                                ));
                            }
                        }
                        on:pointerup=move |event| {
                            event.stop_propagation();
                            if let Some(on_release) = on_release {
                                on_release.run((
                                    f64::from(event.client_x()),
                                    f64::from(event.client_y()),
                                ));
                            }
                        }
                        on:pointercancel=move |_| {
                            if let Some(on_cancel) = on_cancel {
                                on_cancel.run(());
                            }
                        }
                        on:click=move |event| {
                            if swiped.get_untracked() {
                                if let Some(on_swipe_consumed) = on_swipe_consumed {
                                    on_swipe_consumed.run(());
                                }
                                event.stop_propagation();
                            }
                        }
                    />
                </div>

                <Show when=move || many.get()>
                    <button
                        type="button"
                        class="btn btn-circle btn-sm absolute top-1/2 left-2 -translate-y-1/2 cursor-pointer"
                        aria-label="Previous image"
                        on:click=move |event| {
                            event.stop_propagation();
                            if let Some(on_previous) = on_previous {
                                on_previous.run(());
                            }
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
                            if let Some(on_next) = on_next {
                                on_next.run(());
                            }
                        }
                    >
                        "›"
                    </button>
                    <p class="absolute bottom-3 left-1/2 -translate-x-1/2 rounded-box bg-base-300/70 px-2 py-0.5 text-xs">
                        {move || position.get()}
                    </p>
                </Show>

                <Show when=move || !alt.get().is_empty()>
                    <p class="absolute top-3 left-1/2 max-w-[90vw] -translate-x-1/2 rounded-box bg-base-300/70 px-2 py-1 text-center text-xs text-balance">
                        {move || alt.get()}
                    </p>
                </Show>
            </div>
        </Show>
    }
}
