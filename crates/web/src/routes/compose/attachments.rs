//! Attachments on a compose form.
//!
//! The file is read and uploaded as soon as it is picked, so the write on submit only has
//! to reference ids the instance already accepted. That is also why a failed upload never
//! blocks the text: the attachment list is what says whether the post is ready.

use leptos::prelude::*;
use leptos::task::spawn_local;
use nodus_instance::model::MediaAttachment;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

use crate::state::bridge::actions;

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
                            Ok(attachment) => media.update(|list| list.push(attachment)),
                            Err(message) => set_error.set(Some(message)),
                        }
                    }
                    Err(message) => set_error.set(Some(message)),
                }
            }
            set_uploading.set(false);
        });
    };

    view! {
        <div class="flex flex-col gap-2">
            <input
                class="file-input file-input-bordered file-input-sm w-full"
                type="file"
                accept="image/*,video/*,audio/*"
                multiple
                disabled=move || uploading.get()
                on:change=pick
            />
            <Show when=move || uploading.get()>
                <p class="text-xs opacity-60">"Uploading…"</p>
            </Show>
            <ul class="flex flex-wrap gap-2">
                <For each=move || media.get() key=|attachment| attachment.id.clone() let:attachment>
                    {
                        let id = attachment.id.clone();
                        view! {
                            <li class="relative">
                                <img
                                    class="h-20 w-20 rounded-box object-cover"
                                    src=attachment.preview_url.clone().unwrap_or_default()
                                    alt=attachment.description.clone().unwrap_or_default()
                                />
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
