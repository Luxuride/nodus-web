//! Attachments on a compose form.
//!
//! The file is read and uploaded as soon as it is picked, so the write on submit only has
//! to reference ids the instance already accepted. That is also why a failed upload never
//! blocks the text: the attachment list is what says whether the post is ready.

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_instance::model::{MediaAttachment, MediaKind};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

use crate::state::alive::Alive;
use crate::state::bridge::actions;

/// The id of the hidden file input, so the `+` button can open the picker.
const ATTACH_INPUT: &str = "nodus-attach-input";

/// Reads a picked file into bytes and its declared MIME type.
async fn read(file: web_sys::File) -> Result<(Vec<u8>, String), String> {
    let buffer = JsFuture::from(file.array_buffer())
        .await
        .map_err(|error| format!("Could not read the file: {error:?}"))?;
    Ok((js_sys::Uint8Array::new(&buffer).to_vec(), file.type_()))
}

/// The upload control and the thumbnails of what has been uploaded.
#[component]
pub fn Attachments(
    media: RwSignal<Vec<MediaAttachment>>,
    set_error: WriteSignal<Option<String>>,
) -> impl IntoView {
    let (uploading, set_uploading) = signal(false);

    let pick = move |event: leptos::ev::Event| {
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
                        match actions::upload_media(bytes, mime, Some(name)).await {
                            Ok(attachment) => {
                                alive.if_alive(|| media.update(|list| list.push(attachment)));
                            }
                            Err(message) => alive.if_alive(|| set_error.set(Some(message))),
                        }
                    }
                    Err(message) => alive.if_alive(|| set_error.set(Some(message))),
                }
            }
            alive.if_alive(|| set_uploading.set(false));
        });
    };

    // One add button, and the list of what was added. The native control is hidden
    // because its own wording ("No file chosen") is a sentence about the browser's state
    // rather than about the post, and it stays on screen after the upload has finished.
    let open_picker = move |()| {
        if let Some(input) = web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| document.get_element_by_id(ATTACH_INPUT))
            .and_then(|element| element.dyn_into::<web_sys::HtmlInputElement>().ok())
        {
            input.click();
        }
    };

    view! {
        <div class="flex flex-wrap items-center gap-2">
            <input
                id=ATTACH_INPUT
                class="hidden"
                type="file"
                accept="image/*,video/*,audio/*"
                multiple
                disabled=move || uploading.get()
                on:change=pick
            />
            <button
                type="button"
                class="btn btn-outline btn-sm gap-1 border-dashed"
                title="Add images, video or audio"
                disabled=move || uploading.get()
                on:click=move |_| open_picker(())
            >
                <span class="text-base leading-none">"+"</span>
                <span>"Add media"</span>
            </button>
            <Show when=move || uploading.get()>
                <span class="loading loading-spinner loading-xs" aria-label="Uploading"></span>
            </Show>
            <ul class="flex flex-wrap gap-2">
                <For each=move || media.get() key=|attachment| attachment.id.clone() let:attachment>
                    {
                        let id = attachment.id.clone();
                        // Owned before the view: the fallback closure must be `'static`,
                        // and `attachment` is dropped when this iteration ends.
                        // A signal, not a `String`: the fallback closure must be `Fn`,
                        // and a captured `String` would make it `FnOnce`.
                        let (kind_label, _) = signal(format!("{:?}", attachment.kind));
                        let is_image = attachment.kind == MediaKind::Image;
                        view! {
                            <li class="relative">
                                <Show
                                    when=move || is_image
                                    fallback=move || view! {
                                        <span class="badge badge-outline badge-sm">
                                            {kind_label}
                                        </span>
                                    }
                                >
                                    <img
                                        class="h-20 w-20 rounded-box object-cover"
                                        src=attachment.preview_url.clone().unwrap_or_default()
                                        alt=attachment.description.clone().unwrap_or_default()
                                    />
                                </Show>
                                <button
                                    class="btn btn-circle btn-xs absolute -right-1 -top-1"
                                    title="Remove this attachment"
                                    on:click=move |_| {
                                        media.update(|list| list.retain(|item| item.id != id));
                                    }
                                >
                                    "×"
                                </button>
                            </li>
                        }
                    }
                </For>
            </ul>
        </div>
    }
}
