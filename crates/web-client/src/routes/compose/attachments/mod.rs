//! Attachments on a compose form.
//!
//! The file is read and uploaded as soon as it is picked, so the write on submit only has
//! to reference ids the instance already accepted. That is also why a failed upload never
//! blocks the text: the attachment list is what says whether the post is ready.
//!
//! The row of thumbnails is the view's; the file input is not, and neither is the transfer
//! behind it — both are browser objects (`File`, `XmlHttpRequest` progress), so the input
//! arrives at the row as a slot and the tile's two verbs arrive as callbacks.

use std::sync::Arc;

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_instance::model::MediaAttachment;
use nodus_pages::pages::compose::attachments::{
    Attachments as AttachmentList, AttachmentsProps as AttachmentListProps,
};
use nodus_ui::icons::Icon;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

use crate::state::alive::Alive;
use crate::state::bridge::actions;

/// The id of the hidden file input, so the control row can open the picker.
const ATTACH_INPUT: &str = "nodus-attach-input";

/// Reads a picked file into bytes and its declared MIME type.
async fn read(file: web_sys::File) -> Result<(Vec<u8>, String), String> {
    let buffer = JsFuture::from(file.array_buffer())
        .await
        .map_err(|error| format!("Could not read the file: {error:?}"))?;
    Ok((js_sys::Uint8Array::new(&buffer).to_vec(), file.type_()))
}

/// Opens the hidden picker. The button lives in the control row, so the row is the only
/// place that has to know how a file gets chosen.
pub fn request_pick() {
    if let Some(input) = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id(ATTACH_INPUT))
        .and_then(|element| element.dyn_into::<web_sys::HtmlInputElement>().ok())
    {
        input.click();
    }
}

/// The change handler of the hidden input: uploads every picked file, one at a time.
///
/// A free function rather than a closure: the input lives inside a slot, and the slot is
/// called more than once, so the handler cannot be a `FnOnce` that consumes the upload
/// signals.
fn pick(
    event: leptos::ev::Event,
    set_uploading: WriteSignal<bool>,
    set_percent: WriteSignal<Option<u8>>,
    media: RwSignal<Vec<MediaAttachment>>,
    set_error: WriteSignal<Option<String>>,
) {
    let Some(input) = event
        .target()
        .and_then(|target| target.dyn_into::<web_sys::HtmlInputElement>().ok())
    else {
        return;
    };
    let Some(files) = input.files() else {
        return;
    };
    let picked: Vec<web_sys::File> = (0..files.length())
        .filter_map(|index| files.get(index))
        .collect();
    // Clearing the input is what lets the same file be picked twice in a row.
    input.set_value("");
    if picked.is_empty() {
        return;
    }
    let alive = Alive::new();
    set_uploading.set(true);
    set_error.set(None);
    spawn_local(async move {
        for file in picked {
            let name = file.name();
            match read(file).await {
                Ok((bytes, mime)) => {
                    let mime = if mime.is_empty() {
                        "application/octet-stream".to_owned()
                    } else {
                        mime
                    };
                    // The bar has to move while the bytes leave the browser, so the
                    // upload reports each chunk it has sent.
                    alive.if_alive(|| set_percent.set(Some(0)));
                    let progress = {
                        let alive = alive;
                        move |percent: u8| {
                            alive.if_alive(|| {
                                set_percent.set(Some(percent));
                            });
                        }
                    };
                    match actions::upload_media_with_progress(bytes, &mime, Some(&name), progress)
                        .await
                    {
                        Ok(attachment) => {
                            alive.if_alive(|| media.update(|list| list.push(attachment)));
                        }
                        Err(message) => alive.if_alive(|| set_error.set(Some(message))),
                    }
                }
                Err(message) => alive.if_alive(|| set_error.set(Some(message))),
            }
        }
        alive.if_alive(|| {
            set_uploading.set(false);
            set_percent.set(None);
        });
    });
}

/// The hidden file input and the thumbnails of what has been uploaded.
#[component]
pub fn Attachments(
    media: RwSignal<Vec<MediaAttachment>>,
    set_error: WriteSignal<Option<String>>,
) -> impl IntoView {
    let (uploading, set_uploading) = signal(false);
    let (percent, set_percent) = signal(None::<u8>);

    // The input is a slot because it is the browser's, not the view's: `Files`, the accept
    // filter and the click that opens the dialog all belong to this crate.
    let input = Arc::new(move || {
        view! {
            <button
                type="button"
                class="btn btn-outline btn-sm gap-1 border-dashed max-sm:btn-square"
                title="Add media"
                aria-label="Add media"
                disabled=move || uploading.get()
                on:click=move |_| request_pick()
            >
                <Icon icon=Icon::Image/>
                <span class="max-sm:hidden">"Media"</span>
            </button>
            <input
                id=ATTACH_INPUT
                class="hidden"
                type="file"
                accept="image/*,video/*,audio/*"
                multiple
                disabled=move || uploading.get()
                on:change=move |event| pick(event, set_uploading, set_percent, media, set_error)
            />
        }
        .into_any()
    }) as ChildrenFn;

    let remove = Callback::new(move |id: String| {
        media.update(|list| list.retain(|item| item.id != id));
    });

    // Alt text is edited after the upload because that is when the writer knows what the
    // picture is going to say in the post; asking for it before the transfer would be asking
    // about a picture nobody can see yet.
    let describe = Callback::new(move |(id, text): (String, String)| {
        let alive = Alive::new();
        spawn_local(async move {
            match actions::update_media(&id, &text).await {
                Ok(updated) => {
                    media.update(|list| {
                        if let Some(slot) = list.iter_mut().find(|item| item.id == updated.id) {
                            *slot = updated;
                        }
                    });
                }
                Err(message) => alive.if_alive(|| set_error.set(Some(message))),
            }
        });
    });

    AttachmentList(AttachmentListProps {
        media: media.into(),
        uploading: Signal::derive(move || uploading.get()),
        percent: Signal::derive(move || percent.get()),
        on_remove: Some(remove),
        on_alt: Some(describe),
        picker: Some(input),
    })
}
