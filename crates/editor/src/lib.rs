//! The editable body: `ProseMirror`, mounted and spoken to from Rust.
//!
//! Why a JS editor at all: the composer has to show a `:shortcode:` as the picture it will
//! become, mid-sentence, with the caret still in the text. A `textarea` cannot draw an image
//! inside its text, and the workarounds (a mirrored overlay, a preview box below) either
//! drift from the caret or move the target the writer is looking at. `ProseMirror` is the
//! small, honest version of "contenteditable that behaves".
//!
//! What stays in Rust: the draft, the counter, the visibility/warning/language controls and
//! every request. The editor owns a document only while it has focus, and hands back plain
//! text — the only thing the API accepts.

use leptos::prelude::*;
use nodus_instance::model::CustomEmoji;
use wasm_bindgen::JsCast as _;
use wasm_bindgen::closure::Closure;

/// The browser handle to a mounted editor.
#[derive(Debug, Clone)]
struct Handle(js_sys::Object);

thread_local! {
    /// The mounted editor, so a button outside the editor (the emoji picker) can insert
    /// into it. One composer is on screen at a time by design; the last mount wins.
    static ACTIVE: std::cell::RefCell<Option<Handle>> = const { std::cell::RefCell::new(None) };

    /// Scripts already injected, so two composers share one download.
    static SCRIPTS: std::cell::RefCell<Vec<String>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// Loads `url` once and resolves when the global it defines exists.
fn load_script(url: &str, global: &str) -> js_sys::Promise {
    let already = SCRIPTS.with(|seen| seen.borrow().contains(&url.to_owned()));
    let window = web_sys::window().expect("a browser window");
    let loaded =
        js_sys::Reflect::get(&window, &global.into()).is_ok_and(|value| !value.is_undefined());

    if already || loaded {
        return js_sys::Promise::resolve(&wasm_bindgen::JsValue::TRUE);
    }
    SCRIPTS.with(|seen| seen.borrow_mut().push(url.to_owned()));

    let document = window.document().expect("a document");
    let script = document.create_element("script").expect("a script element");
    script.set_attribute("src", url).ok();
    script.set_attribute("defer", "defer").ok();
    let head = document.head().expect("a head");
    let _ = head.append_child(&script);

    // Wait for the load event rather than resolving at once: the file is a bundle, and a
    // composer that mounts before it parses would call a function that does not exist yet.
    js_sys::Promise::new(&mut |resolve, reject| {
        let resolve = resolve.clone();
        let reject = reject.clone();
        let on_load = Closure::once(move || {
            let _ = resolve.call0(&wasm_bindgen::JsValue::UNDEFINED);
        });
        let on_error = Closure::once(move || {
            let _ = reject.call1(
                &wasm_bindgen::JsValue::UNDEFINED,
                &wasm_bindgen::JsValue::from_str("the editor script failed to load"),
            );
        });
        let script: web_sys::HtmlElement = script.clone().unchecked_into();
        script.set_onload(Some(on_load.as_ref().unchecked_ref()));
        script.set_onerror(Some(on_error.as_ref().unchecked_ref()));
        on_load.forget();
        on_error.forget();
    })
}

fn global(path: &str) -> Option<js_sys::Object> {
    let window = web_sys::window()?;
    let mut value: wasm_bindgen::JsValue = window.into();
    for part in path.split('.') {
        value = js_sys::Reflect::get(&value, &part.into()).ok()?;
    }
    value.dyn_into::<js_sys::Object>().ok()
}

fn call_method(
    object: &js_sys::Object,
    method: &str,
    arguments: &[wasm_bindgen::JsValue],
) -> Option<wasm_bindgen::JsValue> {
    let function: js_sys::Function = js_sys::Reflect::get(object, &method.into())
        .ok()?
        .dyn_into()
        .ok()?;
    let args = js_sys::Array::new();
    for argument in arguments {
        args.push(argument);
    }
    function.apply(object, &args).ok()
}

/// The body the writer types into.
///
/// `text` is the draft as Rust sees it; `set_text` receives it back on every change.
/// `emoji` is the catalogue the editor renders and the picker offers — when it arrives
/// after mounting (it is fetched), the editor is told, because a shortcode typed a moment
/// ago should become a picture without a reload.
#[component]
pub fn RichEditor(
    text: ReadSignal<String>,
    set_text: WriteSignal<String>,
    /// The instance's emoji, so a typed `:token:` draws before the post exists.
    emoji: ReadSignal<Vec<CustomEmoji>>,
    /// What to show while the body is empty.
    #[prop(default = "Type your post…".to_owned(), into)]
    placeholder: String,
    /// Enter with the modifier held posts; the editor calls this on Enter alone.
    on_submit: Callback<()>,
    /// The `:token` being completed right now, and the codes that match it.
    ///
    /// The editor knows the catalogue and the document, so it decides what is pending; this
    /// signal is only how that reaches a popup rendered outside the contenteditable.
    query: RwSignal<Option<EmojiQuery>>,
) -> impl IntoView {
    let element = NodeRef::<leptos::html::Div>::new();
    let mounted = StoredValue::new(false);
    // What the editor last told us, so an external `text` change can be told apart from the
    // echo of the writer's own keystroke. Writing that echo back would reset the caret.
    let from_editor = StoredValue::new(String::new());
    let handle = StoredValue::new(None::<Handle>);
    let hint = StoredValue::new(placeholder);

    // Mount once, when the element exists — and keep trying until it does.
    //
    // An `Effect` runs before the view is in the document, so `NodeRef::get` can still be
    // empty on its first pass; reading it is not reactive, so the effect never ran again and
    // the editor never mounted (the buttons around it worked, which is exactly what "the
    // ProseMirror does not work" looked like). A bounded frame loop is the honest fix.
    let boot = std::rc::Rc::new(move || -> bool {
        if mounted.get_value() {
            return true;
        }
        let Some(node) = element.get() else {
            return false;
        };
        // The app mounts into a detached tree and swaps it in (`mount_app`), so a component
        // can exist, and be mountable, before it is in the document. Mounting into a
        // detached node produces an editor nobody can see and no error that says so.
        if !node.is_connected() {
            return false;
        }
        // The bundle is fetched, so the first passes find it missing. `load_script` is
        // idempotent and this loop is the wait: a promise whose load event already fired
        // resolves nothing, which is how the editor used to mount into a page and then
        // silently never appear.
        if global("nodusEditor").is_none() {
            let _ = load_script("/vendor/editor.js", "nodusEditor");
            return false;
        }
        let Some(editor) = global("nodusEditor") else {
            return false;
        };

        let target: web_sys::Element = node.unchecked_into();
        let initial = text.get_untracked();
        let catalogue = emoji.get_untracked();
        from_editor.set_value(initial.clone());

        let on_text = Closure::<dyn FnMut(String)>::new(move |value: String| {
            from_editor.set_value(value.clone());
            // The editor is the source of truth while it has focus; the signal mirrors it
            // for the counter, the post button and the draft.
            set_text.set(value);
        });
        let on_submit = Closure::<dyn FnMut()>::new(move || on_submit.run(()));
        let on_query = query_callback(query);

        let options = mount_options(
            &initial,
            &hint.get_value(),
            &catalogue,
            on_text.as_ref().unchecked_ref(),
            on_submit.as_ref().unchecked_ref(),
            on_query.as_ref().unchecked_ref(),
        );

        let Some(mounted_handle) = call_method(&editor, "mount", &[target.into(), options.into()])
            .and_then(|value| value.dyn_into::<js_sys::Object>().ok())
        else {
            web_sys::console::error_1(&"nodus: mounting the editor failed".into());
            return false;
        };
        ACTIVE.with(|active| *active.borrow_mut() = Some(Handle(mounted_handle.clone())));
        handle.set_value(Some(Handle(mounted_handle)));
        // ProseMirror keeps these for the document's lifetime, so they outlive this frame.
        on_text.forget();
        on_submit.forget();
        on_query.forget();
        mounted.set_value(true);
        true
    });

    let mount_button = when_ready(std::rc::Rc::clone(&boot), handle);

    // External changes (an edit loaded, a form reset) replace the document; the writer's
    // own keystrokes arrive here too and are filtered by comparing with what we were told.
    Effect::new(move |_| {
        let wanted = text.get();
        let Some(editor) = handle.get_value() else {
            return;
        };
        if wanted == from_editor.get_value() {
            return;
        }
        from_editor.set_value(wanted.clone());
        let _ = call_method(&editor.0, "setText", &[wanted.into()]);
    });

    // A catalogue that arrives late still turns the shortcodes already typed into pictures.
    Effect::new(move |_| {
        let catalogue = emoji.get();
        let Some(editor) = handle.get_value() else {
            return;
        };
        let _ = call_method(&editor.0, "setEmoji", &[catalogue_js(&catalogue)]);
    });

    view! {
        // The height is reserved and the typography matches the reader's body, so the
        // composer does not change size when the editor takes over from the first paint.
        <div
            id="nodus-composer-body"
            node_ref=element
            class="min-h-24 w-full cursor-text text-sm leading-relaxed"
            on:mousedown=mount_button
            on:click=move |_| {
                if let Some(editor) = handle.get_value() {
                    let _ = call_method(&editor.0, "focus", &[]);
                } else {
                    let _ = boot();
                }
            }
        ></div>
    }
}

/// The catalogue as the shape the bundle expects: `[{ shortcode, url }]`.
fn catalogue_js(catalogue: &[CustomEmoji]) -> wasm_bindgen::JsValue {
    let array = js_sys::Array::new();
    for emoji in catalogue {
        let entry = js_sys::Object::new();
        let _ = js_sys::Reflect::set(&entry, &"shortcode".into(), &emoji.shortcode.clone().into());
        let source = if emoji.url.is_empty() {
            emoji.static_url.clone().unwrap_or_default()
        } else {
            emoji.url.clone()
        };
        let _ = js_sys::Reflect::set(&entry, &"url".into(), &source.into());
        array.push(&entry);
    }
    array.into()
}

/// Takes the pending suggestion in the mounted editor: `:blob` becomes the emoji.
///
/// The editor owns the token range and the catalogue, so the click comes back here rather
/// than editing the draft text and guessing where the caret was.
pub fn accept_active(code: &str) -> bool {
    ACTIVE.with(|active| {
        let borrowed = active.borrow();
        let Some(handle) = borrowed.as_ref() else {
            return false;
        };
        call_method(&handle.0, "accept", &[code.into()])
            .is_some_and(|value| value.as_bool().unwrap_or(false))
    })
}

/// Inserts text into the composer that is currently on screen.
///
/// Returns `false` when no editor is mounted, so a caller can fall back to appending to the
/// draft instead of silently dropping what the reader picked.
pub fn insert_active(text: &str) -> bool {
    ACTIVE.with(|active| {
        let borrowed = active.borrow();
        let Some(handle) = borrowed.as_ref() else {
            return false;
        };
        call_method(&handle.0, "insert", &[text.into()]).is_some()
    })
}

/// Resolves on the next animation frame, so a wait costs no timer.
async fn next_frame() {
    let promise = js_sys::Promise::new(&mut |resolve, _reject| {
        let Some(window) = web_sys::window() else {
            return;
        };
        let _ = window.request_animation_frame(&resolve);
    });
    let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
}

/// Mounts the editor as soon as both the bundle and the element are available.
///
/// Two triggers and no timer: the `Effect` runs before the element is necessarily in the
/// document, and the timer chain this replaced never fired a second time in production or in
/// the test browser. A press is proof the element is connected, and it is what a writer does
/// before typing anyway.
#[expect(
    clippy::needless_pass_by_value,
    reason = "the guard is shared with the press handler and the load task"
)]
fn when_ready(
    boot: std::rc::Rc<impl Fn() -> bool + 'static>,
    handle: StoredValue<Option<Handle>>,
) -> impl Fn(web_sys::MouseEvent) + 'static {
    let on_press = {
        let boot = boot.clone();
        move |_event: web_sys::MouseEvent| {
            let _ = boot();
        }
    };

    let load_and_mount = {
        let boot = boot.clone();
        move || {
            if handle.get_value().is_some() {
                return;
            }
            let promise = load_script("/vendor/editor.js", "nodusEditor");
            let boot = std::rc::Rc::clone(&boot);
            leptos::task::spawn_local(async move {
                let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
                // The node can still be outside the document on the frame the bundle
                // arrives (`mount_app` swaps a detached tree in), so this waits for it.
                for _ in 0..120 {
                    if boot() {
                        return;
                    }
                    next_frame().await;
                }
                web_sys::console::warn_1(&"nodus: the editor element never appeared".into());
            });
        }
    };
    let on_render = load_and_mount.clone();
    Effect::new(move |_| on_render());
    on_press
}

/// The emoji picker `emoji-picker-element` provides, mounted on demand.
///
/// Kept separate from the composer so opening it costs one fetch, not a bundle: the picker's
/// database is an order of magnitude larger than the editor itself.
#[component]
pub fn EmojiPanel(
    /// Whether the panel is showing; the button that opens it owns the toggle.
    open: ReadSignal<bool>,
    /// The instance's emoji, so the picker offers what this server can render.
    catalogue: ReadSignal<Vec<CustomEmoji>>,
) -> impl IntoView {
    let host = NodeRef::<leptos::html::Div>::new();
    let mounted = StoredValue::new(false);

    Effect::new(move |_| {
        if !open.get() {
            // Closing throws the element away: the picker keeps a sizable DOM and its
            // database is cached by the browser, so rebuilding is cheaper than carrying it.
            if mounted.get_value() {
                if let Some(node) = host.get() {
                    node.set_inner_html("");
                }
                mounted.set_value(false);
            }
            return;
        }
        if mounted.get_value() {
            return;
        }
        let Some(node) = host.get() else {
            return;
        };
        mounted.set_value(true);

        let target: web_sys::Element = node.unchecked_into();
        let catalogue = catalogue.get_untracked();

        let on_pick = Closure::<dyn FnMut(String)>::new(move |text: String| {
            // A unicode emoji is text; a `:shortcode:` becomes an atom inside the editor.
            insert_active(&text);
        });

        let promise = load_script("/vendor/picker.js", "nodusPicker");
        leptos::task::spawn_local(async move {
            let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
            let Some(picker) = global("nodusPicker") else {
                web_sys::console::error_1(&"nodus: the picker bundle did not load".into());
                return;
            };
            let callback: &js_sys::Function = on_pick.as_ref().unchecked_ref();
            let _ = call_method(
                &picker,
                "create",
                &[target.into(), catalogue_js(&catalogue), callback.into()],
            );
            on_pick.forget();
        });
    });

    view! {
        <nodus_ui::Popover
            open=Signal::derive(move || open.get())
            host=host
            class="absolute right-0 top-full z-50 mt-1 w-80 max-w-[calc(100vw-1rem)] rounded-box border border-base-300 bg-base-100 p-2 shadow-xl".to_owned()
        />
    }
}

pub use nodus_pages::pages::compose::inline::hints::{EmojiHint, EmojiHintsValue};

/// The JS callback that carries the pending `:token` and the caret's place in the box.
///
/// It lives outside `RichEditor` so the component stays inside the line budget: the closure
/// body is one wire crossing (a code, an array of pairs, two coordinates) and nothing else.
fn query_callback(
    query: RwSignal<Option<EmojiQuery>>,
) -> Closure<dyn FnMut(String, js_sys::Array, f64, f64, u32)> {
    Closure::<dyn FnMut(String, js_sys::Array, f64, f64, u32)>::new(
        move |code: String, list: js_sys::Array, left: f64, top: f64, active: u32| {
            if code.is_empty() {
                query.set(None);
                return;
            }
            let options = list
                .iter()
                .filter_map(|item| {
                    let pair = item.dyn_into::<js_sys::Array>().ok()?;
                    let code = pair.get(0).as_string()?;
                    let preview = pair.get(1).as_string()?;
                    Some(EmojiHint { code, preview })
                })
                .collect();
            query.set(Some(EmojiHintsValue {
                options,
                left,
                top,
                active: usize::try_from(active).unwrap_or_default(),
            }));
        },
    )
}

/// The `:token` the editor is completing and the shortcodes that match it.
pub type EmojiQuery = EmojiHintsValue;

/// The options object `mount` reads. Kept out of `boot` so that function stays readable.
fn mount_options(
    initial: &str,
    placeholder: &str,
    catalogue: &[CustomEmoji],
    on_text: &js_sys::Function,
    on_submit: &js_sys::Function,
    on_query: &js_sys::Function,
) -> js_sys::Object {
    let options = js_sys::Object::new();
    let _ = js_sys::Reflect::set(&options, &"initial".into(), &initial.into());
    let _ = js_sys::Reflect::set(&options, &"placeholder".into(), &placeholder.into());
    let _ = js_sys::Reflect::set(&options, &"onText".into(), on_text);
    let _ = js_sys::Reflect::set(&options, &"onSubmit".into(), on_submit);
    let _ = js_sys::Reflect::set(&options, &"emoji".into(), &catalogue_js(catalogue));
    let _ = js_sys::Reflect::set(&options, &"onQuery".into(), on_query);
    options
}
