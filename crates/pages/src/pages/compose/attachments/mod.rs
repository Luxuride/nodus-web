//! Attachments on a compose form.
//!
//! The list and the tiles are markup; the file is not. Reading a `File` and posting its
//! bytes is transport, and the hidden `<input type=file>` is the browser's own control, so
//! it arrives as a slot. The colour of the row — "nothing chosen yet" versus "two files
//! here" — is the caller's to decide by what it puts in `media`.

use leptos::prelude::*;
use nodus_instance::model::{MediaAttachment, MediaKind};

/// The uploaded files, and the picker that adds more.
#[component]
pub fn Attachments(
    /// The attachments the instance has already accepted.
    #[prop(into)]
    media: Signal<Vec<MediaAttachment>>,
    /// Whether a transfer is running.
    #[prop(into)]
    uploading: Signal<bool>,
    /// How much of the file has been sent, when the browser can say.
    #[prop(into)]
    percent: Signal<Option<u8>>,
    /// Drops one attachment by id.
    #[prop(optional)]
    on_remove: Option<Callback<String>>,
    /// Saves one attachment's description.
    #[prop(optional)]
    on_alt: Option<Callback<(String, String)>>,
    /// The hidden file input, which owns the browser's file handling.
    #[prop(optional)]
    picker: Option<ChildrenFn>,
) -> impl IntoView {
    view! {
        <div class="flex flex-wrap items-center gap-2">
            {picker.map(|picker| picker())}
            <Show when=move || uploading.get()>
                {
                    move || match percent.get() {
                        Some(value) => view! {
                            <span class="flex items-center gap-1" title="How much of the file has been sent">
                                <progress class="progress progress-primary w-16" value=value max="100"></progress>
                                <span class="text-xs tabular-nums opacity-70">{format!("{value}%")}</span>
                            </span>
                        }.into_any(),
                        None => view! {
                            <span class="loading loading-spinner loading-xs" aria-label="Uploading"></span>
                        }.into_any(),
                    }
                }
            </Show>
            <ul class="flex flex-wrap gap-2">
                <For each=move || media.get() key=|attachment| attachment.id.clone() let:attachment>
                    <AttachmentTile attachment=attachment on_remove=on_remove on_alt=on_alt/>
                </For>
            </ul>
        </div>
    }
}

/// One attachment: what it is, and the alt text that makes it readable to someone who
/// cannot see it.
///
/// Alt text is edited after the upload because that is when the writer knows what the
/// picture is going to say in the post; asking for it before the transfer would be asking
/// about a picture nobody can see yet.
#[component]
fn AttachmentTile(
    attachment: MediaAttachment,
    on_remove: Option<Callback<String>>,
    on_alt: Option<Callback<(String, String)>>,
) -> impl IntoView {
    let remove_id = attachment.id.clone();
    let save_id = attachment.id.clone();
    // A signal, not a `String`: the fallback closure must be `Fn`, and a captured
    // `String` moved into it once would make the whole component `FnOnce`.
    let (label, _) = signal(format!("{:?}", attachment.kind));
    let has_label = attachment
        .description
        .as_deref()
        .is_some_and(|description| !description.trim().is_empty());
    let is_image = attachment.kind == MediaKind::Image;
    let (editing, set_editing) = signal(false);
    let (alt, set_alt) = signal(attachment.description.clone().unwrap_or_default());
    let (label_set, set_label_set) = signal(has_label);

    // `Callback` is `Copy`, so the field and the blur can both ask for the same save.
    let save = Callback::new(move |()| {
        let text = alt.get_untracked();
        if let Some(on_alt) = on_alt {
            on_alt.run((save_id.clone(), text.clone()));
        }
        set_label_set.set(!text.trim().is_empty());
        set_editing.set(false);
    });

    view! {
        <li class="relative flex flex-col gap-1">
            <Show
                when=move || is_image
                fallback=move || {
                    view! { <span class="badge badge-outline badge-sm">{label.get()}</span> }
                }
            >
                <img
                    class="h-20 w-20 rounded-box object-cover"
                    src=attachment.preview_url.clone().unwrap_or_default()
                    alt=attachment.description.clone().unwrap_or_default()
                />
            </Show>

            <button
                type="button"
                class="btn btn-circle btn-xs absolute -right-1 -top-1"
                title="Remove this attachment"
                on:click=move |_| {
                    if let Some(on_remove) = on_remove {
                        on_remove.run(remove_id.clone());
                    }
                }
            >
                "×"
            </button>

            <div class="flex items-center gap-1">
                <button
                    type="button"
                    class="btn btn-ghost btn-xs px-1"
                    title="Describe this attachment for someone who cannot see it"
                    class:text-warning=move || !label_set.get()
                    on:click=move |_| set_editing.update(|open| *open = !*open)
                >
                    {move || if label_set.get() { "Alt ✓" } else { "Alt" }}
                </button>
            </div>

            <Show when=move || editing.get()>
                <input
                    class="input input-bordered input-xs w-40"
                    placeholder="What does it show?"
                    aria-label="Attachment description"
                    prop:value=alt
                    on:input=move |event| set_alt.set(event_target_value(&event))
                    on:keydown=move |event| {
                        #[cfg(target_arch = "wasm32")]
                        if event.key() == "Enter" {
                            event.prevent_default();
                            save.run(());
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        let _ = &event;
                    }
                    on:blur=move |_| save.run(())
                />
            </Show>
        </li>
    }
}
