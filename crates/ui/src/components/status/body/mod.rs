//! The part of a status that is content: warning, text, chips, media and poll.
//!
//! Shared by both paints for the reason the media grid is: this markup used to be built
//! twice, and a class that exists on one paint and not the other is a difference nobody
//! chose. Everything a shared component may not touch — the viewer, the vote transport, the
//! reader's saved preference — arrives as a callback or as an initial state, and its absence
//! is a branch of this markup, never a second layout.
//!
//! The body text arrives already rewritten: `inner_html` is the caller's HTML, so the server
//! hands in the same emoji-substituted string it used to interpolate and the client hands in
//! what the instance sent.

use leptos::prelude::*;
use nodus_client_core::model::{MediaAttachment, Mention, Poll, Status, Tag};

use crate::icons::Icon;
use crate::media::attachments_label;
use crate::media_grid::Picture;
use crate::mfm;

/// A translated body and warning, kept together because the warning is part of the post.
#[derive(Clone, Debug)]
struct Translation {
    content: String,
    spoiler: Option<String>,
}

#[derive(serde::Serialize)]
struct TranslateRequest {
    text: String,
    spoiler: Option<String>,
    source: Option<String>,
    target: String,
}

#[cfg(target_arch = "wasm32")]
#[derive(serde::Deserialize)]
struct TranslateResponse {
    content: String,
    spoiler: Option<String>,
}

#[cfg(target_arch = "wasm32")]
#[derive(serde::Deserialize)]
struct TranslateError {
    error: String,
}

/// Content, media and poll of one status.
///
/// `interactive` is false on the server paint, where media has no viewer to open.
/// `revealed_init` carries the reader's saved preference, because reading it is storage and
/// this crate has none: the caller decides whether the veil starts open.
#[component]
#[expect(
    clippy::needless_pass_by_value,
    clippy::too_many_lines,
    reason = "a Leptos component receives its props by value; the body is one disclosure tree"
)]
pub fn StatusBody(
    /// The status whose content this is.
    status: Status,
    /// Opens the viewer on one picture; absent where nothing can be pressed.
    #[prop(optional)]
    on_open_media: Option<Callback<(Vec<Picture>, usize)>>,
    /// Sends the picked options of a poll; absent where there is nowhere to send them.
    #[prop(optional)]
    on_vote: Option<Callback<(String, Vec<usize>)>>,
    /// Whether media offers a player or a gallery; false for the server paint.
    #[prop(default = true)]
    interactive: bool,
    /// Whether the reader already asked to see sensitive media.
    #[prop(default = false)]
    revealed_init: bool,
    /// The language the reader wants translations in.
    #[prop(default = "en".to_owned())]
    target_language: String,
) -> impl IntoView {
    // Per status, not global: a reader who opens one post's warning has not consented to the
    // next one, and the flag is a claim about this attachment set only.
    let (revealed, set_revealed) = signal(revealed_init);
    let (translated, set_translated) = signal(Option::<Translation>::None);
    let (translating, set_translating) = signal(false);
    let (translation_error, set_translation_error) = signal(Option::<String>::None);

    let sensitive = status.sensitive;
    let spoiler = StoredValue::new(status.spoiler_text.trim().to_owned());
    let warned = !status.spoiler_text.trim().is_empty();
    // A warning hides the post the same way sensitivity does. Leaving the text behind a
    // <details> while the media grid stayed below it showed the part most people use a
    // warning for before they had consented to it.
    let hidden = move || sensitive && !revealed.get();
    let opened = move || !warned || revealed.get();

    let content = status.content.clone();
    let source = status.content_source.clone();
    let format = status.content_format.clone();
    let emojis = status.emojis.clone();
    let translated_for_body = translated;
    let body_html = StoredValue::new(move || {
        if let Some(translation) = translated_for_body.get() {
            return with_emojis(&translation.content, &emojis);
        }
        if format.as_deref() == Some("mfm") {
            mfm::render(source.as_deref().unwrap_or(&content), &emojis)
        } else {
            with_emojis(&content, &emojis)
        }
    });
    let spoiler_text = StoredValue::new(move || {
        translated
            .get()
            .and_then(|translation| translation.spoiler)
            .unwrap_or_else(|| spoiler.get_value())
    });

    let language = status.language.clone();
    let target = StoredValue::new(target_language);
    let has_text = !status.content.is_empty() || !status.spoiler_text.trim().is_empty();
    let can_translate = language.as_deref().is_some_and(|source| {
        !source.trim().is_empty() && !same_language(source, &target.get_value())
    }) && has_text;
    let request_text = status.content.clone();
    let request_spoiler =
        (!status.spoiler_text.trim().is_empty()).then(|| status.spoiler_text.trim().to_owned());
    let request_source = language.clone();
    let start_translation = Callback::new(move |()| {
        if translated.get_untracked().is_some() {
            set_translated.set(None);
            set_translation_error.set(None);
            return;
        }
        if translating.get_untracked() {
            return;
        }
        let request = TranslateRequest {
            text: request_text.clone(),
            spoiler: request_spoiler.clone(),
            source: request_source.clone(),
            target: target.get_value(),
        };
        set_translating.set(true);
        set_translation_error.set(None);
        #[cfg(target_arch = "wasm32")]
        {
            use gloo_net::http::Request;
            leptos::task::spawn_local(async move {
                let response = match Request::post("/public-api/translate")
                    .json(&request)
                    .map_err(|error| error.to_string())
                {
                    Ok(request) => request.send().await.map_err(|error| error.to_string()),
                    Err(error) => Err(error),
                };
                let result = match response {
                    Ok(response) if response.ok() => response
                        .json::<TranslateResponse>()
                        .await
                        .map_err(|error| error.to_string()),
                    Ok(response) => match response.json::<TranslateError>().await {
                        Ok(error) => Err(error.error),
                        Err(_) => Err(format!(
                            "The translation service answered {}.",
                            response.status()
                        )),
                    },
                    Err(error) => Err(format!("Could not reach the translator: {error}")),
                };
                match result {
                    Ok(translation) => set_translated.set(Some(Translation {
                        content: translation.content,
                        spoiler: translation.spoiler,
                    })),
                    Err(error) => set_translation_error.set(Some(error)),
                }
                set_translating.set(false);
            });
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = request;
            set_translating.set(false);
        }
    });

    let attachments = status.media_attachments.clone();
    let items = Signal::derive(move || attachments.clone());
    let has_media = !status.media_attachments.is_empty();
    let veil_label = StoredValue::new(attachments_label(&status));
    let poll = StoredValue::new(status.poll.clone());

    view! {
        <Show when=move || warned>
            <div class="rounded-box bg-base-200 p-3">
                <p class="text-sm font-medium">{move || spoiler_text.with_value(|text| text())}</p>
                <div class="mt-2 flex flex-wrap items-center gap-2">
                    <button
                        type="button"
                        class="btn btn-ghost btn-xs"
                        on:click=move |_| set_revealed.update(|revealed| *revealed = !*revealed)
                    >
                        {move || if revealed.get() { "Hide content" } else { "Show content" }}
                    </button>
                    {translation_control(
                        can_translate,
                        translating,
                        translated,
                        translation_error,
                        start_translation,
                    )}
                </div>
                <Show when=move || revealed.get()>
                    <p class="mt-2 whitespace-pre-line text-sm leading-relaxed">
                        <span inner_html=move || body_html.with_value(|html| html())></span>
                    </p>
                </Show>
            </div>
        </Show>
        <Show when=move || !warned>
            <p class="whitespace-pre-line text-sm leading-relaxed">
                <span inner_html=move || body_html.with_value(|html| html())></span>
            </p>
            {translation_control(
                can_translate,
                translating,
                translated,
                translation_error,
                start_translation,
            )}
        </Show>

        <StatusChips
            tags=status.tags.clone()
            mentions=status.mentions.clone()
            url=status.url.clone()
        />

        <Show when=move || opened()>
            <Show when=move || hidden()>
                // Blurred *and* labelled: a blur alone leaves the reader guessing whether the
                // hidden thing is a photo, a chart or a joke, which is worse than a warning.
                <div class="relative overflow-hidden rounded-box">
                    {grid(items, true, interactive, on_open_media)}
                    // Only a label and a click target: the sharpest thing on screen should not
                    // be the overlay, and a veil over a blur reads as a bug rather than a gate.
                    <button
                        type="button"
                        class="absolute inset-0 flex cursor-pointer items-center justify-center bg-transparent"
                        on:click=move |_| set_revealed.set(true)
                    >
                        <span class="rounded-box bg-base-300/90 px-3 py-1.5 text-center shadow-sm transition-transform hover:scale-105">
                            <span class="block text-sm font-medium">"Sensitive content"</span>
                            <span class="block text-xs opacity-70">{move || veil_label.get_value()}</span>
                        </span>
                    </button>
                </div>
            </Show>

            <Show when=move || has_media && !hidden()>
                {grid(items, false, interactive, on_open_media)}
            </Show>

            {move || {
                poll.with_value(|poll| {
                    poll.clone()
                        .map(|poll| PollBlock(PollBlockProps { poll, on_vote }))
                })
            }}
        </Show>
    }
}

/// The one control that asks for a translation, drawn in both branches of a post.
fn translation_control(
    can_translate: bool,
    translating: ReadSignal<bool>,
    translated: ReadSignal<Option<Translation>>,
    error: ReadSignal<Option<String>>,
    start: Callback<()>,
) -> AnyView {
    view! {
        <Show when=move || can_translate>
            <div class="flex flex-col gap-1">
                <button
                    type="button"
                    class="btn btn-ghost btn-xs"
                    disabled=move || translating.get()
                    on:click=move |_| start.run(())
                >
                    {move || match (translating.get(), translated.get().is_some()) {
                        (true, _) => "Translating...".to_owned(),
                        (false, true) => "Show original".to_owned(),
                        (false, false) => "Translate".to_owned(),
                    }}
                </button>
                <Show when=move || error.get().is_some()>
                    <span class="text-xs text-error">{move || error.get().unwrap_or_default()}</span>
                </Show>
            </div>
        </Show>
    }
    .into_any()
}

/// Whether two codes name the same language. `en-US` and `en` do, and a reader who set
/// `pt-BR` should not be offered a translation of every `pt` post.
fn same_language(left: &str, right: &str) -> bool {
    let primary = |code: &str| {
        code.trim()
            .split(['-', '_'])
            .next()
            .unwrap_or_default()
            .to_ascii_lowercase()
    };
    primary(left) == primary(right)
}

/// One attachment grid, with the viewer callback forwarded exactly as it arrived.
///
/// The props are built as a struct rather than through `view!` because `on_open` is optional:
/// its setter takes the bare callback, and a component that only has the option to forward
/// would have to invent one.
fn grid(
    items: Signal<Vec<MediaAttachment>>,
    hidden: bool,
    interactive: bool,
    on_open: Option<Callback<(Vec<Picture>, usize)>>,
) -> AnyView {
    crate::media_grid::MediaGrid(crate::media_grid::MediaGridProps {
        items,
        hidden,
        interactive,
        on_open,
    })
    .into_any()
}

/// Replace `:shortcode:` tokens with the instance's emoji images.
///
/// The server already does this while preparing its HTML; the browser receives the raw
/// shortcodes from Mastodon, so the shared body has to paint them there too.
fn with_emojis(content: &str, emojis: &[nodus_client_core::model::CustomEmoji]) -> String {
    if emojis.is_empty() {
        return content.to_owned();
    }
    let mut painted = content.to_owned();
    for emoji in emojis {
        let source = if emoji.url.is_empty() {
            emoji.static_url.as_deref().unwrap_or_default()
        } else {
            emoji.url.as_str()
        };
        if source.is_empty() {
            continue;
        }
        let token = format!(":{}:", emoji.shortcode);
        // The server already prepared this HTML: its emoji image carries the shortcode in
        // `alt`/`title`, so replacing the token again would nest markup inside the attribute.
        if painted.contains(&format!("alt=\"{token}\"")) {
            continue;
        }
        let replacement = format!(
            "<img class=\"emoji inline-block h-[1.2em] w-auto align-[-0.2em]\" alt=\"{token}\" title=\"{token}\" loading=\"lazy\" decoding=\"async\" src=\"{source}\">",
            token = token
                .replace('&', "&amp;")
                .replace('"', "&quot;")
                .replace('<', "&lt;"),
            source = source
                .replace('&', "&amp;")
                .replace('"', "&quot;")
                .replace('<', "&lt;"),
        );
        painted = painted.replace(&token, &replacement);
    }
    painted
}

/// Hashtags, mentions and the permalink, in that order.
#[component]
fn StatusChips(
    /// The hashtags the instance found in the content.
    tags: Vec<Tag>,
    /// The accounts the instance found in the content.
    mentions: Vec<Mention>,
    /// The permalink, when the post is public enough to have one.
    url: Option<String>,
) -> impl IntoView {
    view! {
        <div class="flex flex-wrap items-center gap-1.5">
            <For each=move || tags.clone() key=|tag| format!("tag-{}", tag.name) let:tag>
                {
                    let href = format!("/search?q={}", encode(&format!("#{}", tag.name)));
                    view! {
                        <a class="badge badge-outline badge-sm gap-1" href=href title="Search this hashtag">
                            <Icon icon=Icon::Tag class="size-3"/>
                            {format!("#{}", tag.name)}
                        </a>
                    }
                }
            </For>
            <For
                each=move || mentions.clone()
                key=|mention| format!("mention-{}", mention.acct)
                let:mention
            >
                {
                    let href = format!("/search?q={}", encode(&format!("@{}", mention.acct)));
                    view! {
                        <a class="badge badge-ghost badge-sm gap-1" href=href title="Search this account">
                            <Icon icon=Icon::At class="size-3"/>
                            {format!("@{}", mention.acct)}
                        </a>
                    }
                }
            </For>
            {url.map(|url| view! {
                <a
                    class="ml-auto flex items-center gap-1 text-xs opacity-60 hover:opacity-100"
                    href=url
                    target="_blank"
                    rel="nofollow noopener noreferrer"
                    title="Open this post on its own instance"
                >
                    <Icon icon=Icon::Link class="size-3"/>
                    "Original"
                </a>
            })}
        </div>
    }
}

/// Percent-encodes what a query string cannot carry, so `#rust` is not read as a fragment.
fn encode(text: &str) -> String {
    use std::fmt::Write as _;

    let mut encoded = String::with_capacity(text.len() + 4);
    for byte in text.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(char::from(byte));
            }
            _ => {
                let _ = write!(encoded, "%{byte:02X}");
            }
        }
    }
    encoded
}

/// A poll: one row per option and the button that sends the pick.
///
/// The picked options are this component's own state, because picking is not a status
/// change. With no `on_vote` every row and the button keep their box and their bars and are
/// disabled instead: the server paints the same poll a signed-out reader sees.
#[component]
fn PollBlock(
    /// The poll to draw.
    poll: Poll,
    /// Sends the picked options; absent where there is nowhere to send them.
    #[prop(optional)]
    on_vote: Option<Callback<(String, Vec<usize>)>>,
) -> impl IntoView {
    let Poll {
        id,
        multiple,
        expired,
        voted,
        votes_count,
        options,
        ..
    } = poll;
    let (picked, set_picked) = signal(Vec::<usize>::new());
    let (error, set_error) = signal(None::<String>);
    // A closed, already-voted or unreachable poll stays pressable nowhere: the instance
    // refuses it, and a button that cannot work should not look like one.
    let locked = on_vote.is_none() || voted || expired;
    let total: u64 = options.iter().filter_map(|option| option.votes_count).sum();
    let tally = if expired {
        "Poll closed".to_owned()
    } else {
        format!("{votes_count} votes")
    };
    let id = StoredValue::new(id);
    let choose = move |index: usize| {
        set_picked.update(|picked| {
            if !multiple {
                picked.clear();
                picked.push(index);
                return;
            }
            if let Some(position) = picked.iter().position(|choice| *choice == index) {
                picked.remove(position);
            } else {
                picked.push(index);
            }
        });
    };

    let vote = move |()| {
        let Some(on_vote) = on_vote else {
            return;
        };
        let choices = picked.get_untracked();
        if choices.is_empty() {
            set_error.set(Some("Pick an option first.".to_owned()));
            return;
        }
        set_error.set(None);
        on_vote.run((id.get_value(), choices));
    };

    // Collected outside the macro: a turbofish inside `view!` ends the tag early.
    let rows = move || options.clone().into_iter().enumerate().collect::<Vec<_>>();

    view! {
        <div class="rounded-box border border-base-300 p-3">
            <For
                each=rows
                key=|(index, option)| (*index, option.title.clone())
                let:entry
            >
                {
                    let (index, option) = entry;
                    view! {
                        <PollOption
                            index=index
                            title=option.title
                            votes=option.votes_count.unwrap_or(0)
                            total=total
                            locked=locked
                            picked=picked
                            pick=Callback::new(choose)
                        />
                    }
                }
            </For>
            <div class="flex items-center justify-between gap-2">
                <span class="text-xs opacity-60">{tally}</span>
                <button class="btn btn-primary btn-sm" disabled=locked on:click=move |_| vote(())>
                    "Vote"
                </button>
            </div>
            <Show when=move || error.get().is_some()>
                <p class="text-error mt-1 text-xs">{move || error.get().unwrap_or_default()}</p>
            </Show>
        </div>
    }
}

/// One option line, with a bar showing its share.
#[component]
fn PollOption(
    /// Where the option sits in the poll.
    index: usize,
    /// The label, plain text.
    title: String,
    /// Votes for it; zero when the instance withholds counts.
    votes: u64,
    /// Votes for the whole poll, the denominator of the bar.
    total: u64,
    /// Whether the row can be pressed at all.
    locked: bool,
    /// The options the reader has picked.
    picked: ReadSignal<Vec<usize>>,
    /// Picks or unpicks this option.
    pick: Callback<usize>,
) -> impl IntoView {
    // `checked_div` rather than a guard: a poll with no votes at all has no share, and the
    // expression says so without a branch that a later edit could invert.
    let share = votes
        .checked_mul(100)
        .and_then(|scaled| scaled.checked_div(total))
        .unwrap_or(0);

    view! {
        <button
            class="mb-2 w-full rounded-box border border-base-300 px-3 py-2 text-left text-sm hover:border-primary"
            class:border-primary=move || picked.get().contains(&index)
            disabled=locked
            on:click=move |_| pick.run(index)
        >
            <span class="flex items-center justify-between gap-2">
                <span>{title}</span>
                <span class="text-xs opacity-60">{format!("{votes} · {share}%")}</span>
            </span>
            <span class="mt-1 block h-1 w-full rounded bg-base-300">
                <span class="block h-1 rounded bg-primary" style=format!("width: {share}%")></span>
            </span>
        </button>
    }
}
