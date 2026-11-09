//! The attachment grid, shared by the server paint and the client mount.
//!
//! The grid decides *where* everything below a post sits, so it cannot be derived twice:
//! a tile with a published size reserves that height, a tile without one reserves 16:9,
//! and a blurred tile keeps the same box as the sharp one it replaces. `crate::media` owns
//! those rules; this module is the view over them.
//!
//! `interactive` is the whole difference between the two paints. On the server a picture is
//! a picture (no button, no player, no gallery position), because there is no way to press
//! it yet; the markup, classes and reserved boxes are otherwise identical, so the client
//! mount does not move anything.

use leptos::prelude::*;
use nodus_instance::model::{MediaAttachment, MediaKind};

/// One picture, in the shape a viewer needs it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Picture {
    /// The full-size file.
    pub url: String,
    /// The author's description, used as the caption and the alt text.
    pub alt: String,
}

/// The pictures of one status, in order, which is the order a gallery opens in.
#[must_use]
pub fn pictures(items: &[MediaAttachment]) -> Vec<Picture> {
    items
        .iter()
        .filter(|media| media.kind == MediaKind::Image)
        .map(|media| Picture {
            url: media.url.clone().unwrap_or_default(),
            alt: media.description.clone().unwrap_or_default(),
        })
        .collect()
}

/// The height a tile reserves, as an inline style, so a hidden and a revealed picture
/// occupy the same box.
pub(crate) fn veil(blurred: bool) -> &'static str {
    crate::media::veil(blurred)
}

/// Attachment tiles for one status.
///
/// `on_open` receives the gallery and the position the reader pressed. Absent on the
/// server, where nothing can be pressed.
#[component]
pub fn MediaGrid(
    items: Signal<Vec<MediaAttachment>>,
    hidden: bool,
    /// Whether the tiles offer a player or a gallery; false for the server paint.
    #[prop(default = true)]
    interactive: bool,
    #[prop(optional)] on_open: Option<Callback<(Vec<Picture>, usize)>>,
) -> impl IntoView {
    // Collected outside the macro: a turbofish inside `view!` ends the tag early.
    let rows = move || items.get().into_iter().enumerate().collect::<Vec<_>>();
    // The grid must be exactly as wide as the pictures it holds, or a centred label lands
    // in the gap beside a lone image.
    let columns = move || {
        if items.get().len() == 1 {
            "grid grid-cols-1 items-start gap-2"
        } else {
            "grid grid-cols-2 items-start gap-2"
        }
    };

    view! {
        <ul class=columns aria-hidden=hidden.to_string()>
            <For each=rows key=|(_, media)| media.id.clone() let:entry>
                <MediaTile items=items media=entry.1 interactive=interactive blurred=hidden on_open=on_open/>
            </For>
        </ul>
    }
}

/// One attachment.
///
/// The strings live in signals rather than locals because `Show`'s children closure must
/// be `Fn`: a `String` captured by value makes it `FnOnce`, and a signal is `Copy`.
#[component]
fn MediaTile(
    items: Signal<Vec<MediaAttachment>>,
    media: MediaAttachment,
    interactive: bool,
    blurred: bool,
    on_open: Option<Callback<(Vec<Picture>, usize)>>,
) -> impl IntoView {
    let is_image = media.kind == MediaKind::Image;
    let kind = media.kind;
    // Video and audio are played in place: a link that opens a tab is a worse answer than
    // a player, and the instance already served a poster frame for the video case.
    let plays = matches!(kind, MediaKind::Video | MediaKind::Gifv);
    let listens = kind == MediaKind::Audio;
    let id = StoredValue::new(media.id.clone());
    let (url, _) = signal(media.url.clone().unwrap_or_default());
    let (preview, _) = signal(media.preview_url.clone().unwrap_or_default());
    let (description, _) = signal(media.description.clone().unwrap_or_default());
    // The instance reports the original's size, and the preview keeps the same ratio, so
    // declaring it reserves the row before a byte arrives. Without it every image load
    // pushed the page down by its own height, which is what "the website jumps" means.
    let (width, _) = signal(media.meta_width);
    let (height, _) = signal(media.meta_height);
    // A `<video>` element owns a decoder from the moment it exists, even with
    // `preload="metadata"`, and a timeline of a hundred posts mounted a hundred of them:
    // the tab grew until scrolling stuttered. A video sits behind its poster until asked.
    let (playing, set_playing) = signal(false);
    let (listening, set_listening) = signal(false);

    // A tile with no published size still reserves its box: the fallback is the shared 16:9
    // rule, never "as tall as the file turns out to be".
    let box_ = move || crate::media::style_for(width.get(), height.get());

    // The click belongs to the picture: the card's own click-to-open handler must not see
    // the same press, or the reader gets the thread instead of the enlargement.
    let open_viewer = move |event: leptos::ev::MouseEvent| {
        // The DOM methods exist only on the browser target; the server never renders this
        // branch (`interactive` is false there), so the event is merely unused in that build.
        #[cfg(target_arch = "wasm32")]
        {
            event.prevent_default();
            event.stop_propagation();
        }
        #[cfg(not(target_arch = "wasm32"))]
        let _ = &event;
        let Some(on_open) = on_open else { return };
        let all = items.get_untracked();
        let position = all
            .iter()
            .filter(|media| media.kind == MediaKind::Image)
            .position(|media| media.id == id.get_value())
            .unwrap_or(0);
        on_open.run((pictures(&all), position));
    };

    let veil = veil(blurred);

    view! {
        <li class="self-start overflow-hidden rounded-box">
            <Show when=move || is_image && interactive>
                <button type="button" class="block w-full cursor-zoom-in" on:click=open_viewer>
                    // The aspect box is what keeps a blurred picture and a sharp one the
                    // same height: the veil scales the image to hide its soft edges, and
                    // scaling an image that is itself the layout box moves everything below.
                    <div class="relative w-full overflow-hidden rounded-box bg-base-300" style=box_>
                        <img
                            class="absolute inset-0 h-full w-full object-cover"
                            style=veil
                            src=move || preview.get()
                            alt=move || description.get()
                            title=move || description.get()
                            width=move || width.get().map(|width| width.to_string()).unwrap_or_default()
                            height=move || height.get().map(|height| height.to_string()).unwrap_or_default()
                            loading="lazy"
                            decoding="async"
                            referrerpolicy="no-referrer"
                        />
                    </div>
                </button>
            </Show>
            <Show when=move || is_image && !interactive>
                <img
                    class="h-auto w-full rounded-box bg-base-300 object-cover"
                    class:aspect-video=move || width.get().is_none()
                    style=veil
                    src=move || preview.get()
                    alt=move || description.get()
                    width=move || width.get().map(|width| width.to_string()).unwrap_or_default()
                    height=move || height.get().map(|height| height.to_string()).unwrap_or_default()
                    loading="lazy"
                    decoding="async"
                    referrerpolicy="no-referrer"
                />
            </Show>
            <Show when=move || plays && interactive && playing.get()>
                <video
                    class="max-h-[32rem] w-full rounded-box bg-base-300"
                    style=veil
                    controls
                    autoplay
                    playsinline
                    preload="metadata"
                    poster=move || preview.get()
                    src=move || url.get()
                    aria-label=move || description.get()
                    crossorigin="anonymous"
                ></video>
                <p class="mt-1 flex flex-wrap gap-x-3 text-xs opacity-60">
                    <span>{move || description.get()}</span>
                    <a
                        class="link link-hover"
                        href=move || url.get()
                        target="_blank"
                        rel="nofollow noopener noreferrer"
                    >
                        "Open original"
                    </a>
                </p>
            </Show>
            <Show when=move || plays && interactive && !playing.get()>
                <button
                    type="button"
                    class="relative block w-full cursor-pointer"
                    title="Play this video"
                    on:click=move |_| set_playing.set(true)
                >
                    <img
                        class="max-h-[32rem] w-full rounded-box bg-base-300 object-cover"
                        style=veil
                        src=move || preview.get()
                        alt=move || description.get()
                        loading="lazy"
                        decoding="async"
                        referrerpolicy="no-referrer"
                    />
                    <span class="absolute inset-0 flex items-center justify-center">
                        <span class="flex h-14 w-14 items-center justify-center rounded-full bg-base-100/80 text-xl">
                            "▶"
                        </span>
                    </span>
                </button>
            </Show>
            <Show when=move || plays && !interactive>
                <video
                    class="max-h-[32rem] w-full rounded-box bg-base-300"
                    style=veil
                    controls
                    playsinline
                    preload="metadata"
                    loop
                    poster=move || preview.get()
                    src=move || url.get()
                ></video>
            </Show>
            <Show when=move || listens && interactive && listening.get()>
                <audio class="w-full" controls preload="metadata" src=move || url.get()></audio>
            </Show>
            <Show when=move || listens && interactive && !listening.get()>
                <button
                    type="button"
                    class="btn btn-outline btn-sm gap-2"
                    on:click=move |_| set_listening.set(true)
                >
                    "▶ Play audio"
                </button>
            </Show>
            <Show when=move || listens && !interactive>
                <audio class="w-full" controls preload="metadata" src=move || url.get()></audio>
            </Show>
            <Show when=move || !is_image && !plays && !listens && interactive>
                <a
                    class="btn btn-ghost btn-sm w-full"
                    href=move || url.get()
                    target="_blank"
                    rel="nofollow noopener noreferrer"
                    referrerpolicy="no-referrer"
                >
                    {format!("Open {kind:?}")}
                </a>
            </Show>
        </li>
    }
}
