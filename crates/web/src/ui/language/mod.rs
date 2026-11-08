//! The language picker, shared by every composer.
//!
//! The list comes from the instance, not from a table in this crate: an instance may switch
//! its published languages off or add regional ones, and a picker that offered a code the
//! server does not accept would be a button that always fails. When the instance publishes
//! nothing, a short list of common codes keeps the control usable instead of empty.
//!
//! It looks like the visibility picker beside it — same border, same height, `w-fit` — but it
//! is a popover rather than a `<select>`, because a native select cannot be filtered and the
//! list is long enough that typing beats scrolling. The first entry is preselected: Mastodon
//! always sends a language, and a post with no language is guessed from text that may be a
//! quotation.

use leptos::prelude::*;
use nodus_instance::model::Language;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;

use crate::state::bridge::actions;

mod list;

use list::LanguageList;

type ClickListener = Closure<dyn FnMut(web_sys::MouseEvent)>;

thread_local! {
    /// The one document listener, for the same reason as the emoji popover: a `Closure` is
    /// neither `Send` nor `Sync`, so it cannot live in a signal.
    static LISTENER: std::cell::RefCell<Option<ClickListener>> = const { std::cell::RefCell::new(None) };
}

/// Codes offered when the instance publishes no list at all.
const FALLBACK: [(&str, &str); 12] = [
    ("en", "English"),
    ("de", "Deutsch"),
    ("fr", "Français"),
    ("es", "Español"),
    ("it", "Italiano"),
    ("nl", "Nederlands"),
    ("pl", "Polski"),
    ("pt", "Português"),
    ("cs", "Čeština"),
    ("ja", "日本語"),
    ("zh", "中文"),
    ("ru", "Русский"),
];

/// Chooses the language a post is written in.
///
/// An empty value means "the instance decides"; that only survives until the list arrives,
/// after which the first entry is selected — the common case costs no choice, and the value
/// that goes out is one the instance published.
#[component]
pub fn LanguagePicker(
    language: ReadSignal<String>,
    set_language: WriteSignal<String>,
) -> impl IntoView {
    let (options, set_options) = signal(Vec::<Language>::new());
    let (open, set_open) = signal(false);
    let host = NodeRef::<leptos::html::Div>::new();
    let alive = crate::state::alive::Alive::new();

    leptos::task::spawn_local(async move {
        // Two reads: the catalogue is what the list shows, the account's own choice is what
        // it should start on. Both are cached for the document, so the second open is free.
        let fetched = actions::languages().await;
        let preferred = actions::my_language().await;
        alive.if_alive(|| {
            let ordered = ordered_with_default(fetched, preferred.as_deref(), &mut |default| {
                if language.get_untracked().is_empty() {
                    set_language.set(default);
                }
            });
            set_options.set(ordered);
        });
    });

    let name_of = move |code: &str| {
        options
            .get()
            .iter()
            .find(|language| language.code == code)
            .map_or_else(|| "Language".to_owned(), |language| language.name.clone())
    };

    // A press anywhere else puts the list away, the same rule as the emoji popover.
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
                    if !inside {
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

    // `Callback` rather than a closure, because the same handler serves the list and the
    // Enter key in the filter, and a `FnOnce` closure cannot be handed to both.
    // An empty code is the panel's Escape: close without changing the language.
    let pick = Callback::new(move |code: String| {
        if !code.is_empty() {
            set_language.set(code);
        }
        set_open.set(false);
    });
    let selected = language;

    view! {
        // Same footprint as the visibility select next to it, so the bar reads as one row of
        // controls rather than a select plus a stray button.
        <div class="relative" node_ref=host>
            <button
                type="button"
                // `bg-none` kills the arrow daisyUI paints on `.select`; without it the button
                // shows that background caret *and* the span below, which reads as two.
                // `w-fit` lets the control follow the chosen name instead of a fixed width.
                class="select select-bordered select-sm w-fit max-w-72 gap-1 bg-none pr-2 text-left"
                aria-haspopup="listbox"
                aria-expanded=move || open.get().to_string()
                aria-label="Post language"
                title="The language this post is written in"
                on:click=move |_| set_open.update(|showing| *showing = !*showing)
            >
                // No truncation: the point of the wider control is that "Português (Brasil)"
                // is readable without opening the list.
                <span class="whitespace-nowrap">{move || name_of(&language.get())}</span>
                // Same colour as the text: the visibility select's native caret is not faded, and a
                // washed-out arrow next to a solid one reads as a disabled control.
                <span class="text-[0.6rem] leading-none">"▼"</span>
            </button>

            <Show when=move || open.get()>
                // An Escape hands `pick` an empty code: the panel closes, the choice stays.
                <LanguageList options=options selected=selected on_pick=pick/>
            </Show>
        </div>
    }
}

/// Orders the catalogue with the instance default first, and reports that default.
///
/// The instance's own first language is the default: it is selected and shown at the top, so
/// the common case costs no choice and a writer never has to scroll to find it. Everything
/// else follows alphabetically.
fn ordered_with_default(
    fetched: Vec<Language>,
    preferred: Option<&str>,
    select: &mut impl FnMut(String),
) -> Vec<Language> {
    let mut list = if fetched.is_empty() {
        FALLBACK
            .iter()
            .map(|(code, name)| Language {
                code: (*code).to_owned(),
                name: (*name).to_owned(),
            })
            .collect::<Vec<_>>()
    } else {
        fetched
    };
    // The account's own language wins; otherwise English, which is what Mastodon falls back
    // to when nothing names a default; otherwise whatever the instance published first.
    let codes: Vec<String> = list.iter().map(|language| language.code.clone()).collect();
    let default = preferred
        .filter(|code| codes.iter().any(|known| known == code))
        .map(str::to_owned)
        .or_else(|| codes.iter().find(|code| code.as_str() == "en").cloned())
        .or_else(|| codes.first().cloned());
    list.sort_by(|left, right| {
        let rank = |language: &Language| u8::from(Some(&language.code) != default.as_ref());
        rank(left)
            .cmp(&rank(right))
            .then_with(|| left.name.cmp(&right.name))
    });
    if let Some(default) = default {
        select(default);
    }
    list
}

/// The human name of a language code, or the code itself.
///
/// Used where a code is shown outside the picker (the composer's success line): a writer
/// who chose "Deutsch" should not be told they posted in "de".
#[must_use]
pub fn name_of(code: &str) -> String {
    FALLBACK
        .iter()
        .find(|(known, _)| *known == code)
        .map_or_else(|| code.to_owned(), |(_, name)| (*name).to_owned())
}
