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
use crate::mfm;
use crate::media::attachments_label;
use crate::media_grid::Picture;

/// Content, media and poll of one status.
///
/// `interactive` is false on the server paint, where media has no viewer to open.
/// `revealed_init` carries the reader's saved preference, because reading it is storage and
/// this crate has none: the caller decides whether the veil starts open.
#[component]
#[expect(
    clippy::needless_pass_by_value,
    reason = "a Leptos component receives its props by value"
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
) -> impl IntoView {
    // Per status, not global: a reader who opens one post's warning has not consented to the
    // next one, and the flag is a claim about this attachment set only.
    let (revealed, set_revealed) = signal(revealed_init);
    let sensitive = status.sensitive;
    let hidden = move || sensitive && !revealed.get();
    let spoiler = status.spoiler_text.trim().to_owned();
    let warned = !spoiler.is_empty();
    // Two copies, one per branch: the branches are separate closures, and a view is built
    // from the text inside each of them.
    let warned_content = status.content.clone();
    let plain_content = status.content.clone();
    let warned_source = status.content_source.clone();
    let plain_source = status.content_source.clone();
    let warned_format = status.content_format.clone();
    let plain_format = status.content_format.clone();
    let warned_emojis = status.emojis.clone();
    let plain_emojis = status.emojis.clone();
    // One copy for both grids: the signal is what lets the grid rebuild when the card's
    // copy of the status is replaced, while this component stays mounted.
    let attachments = status.media_attachments.clone();
    let items = Signal::derive(move || attachments.clone());
    let has_media = !status.media_attachments.is_empty();
    let veil_label = attachments_label(&status);
    let poll = status.poll.clone();

    view! {
        <Show when=move || warned>
            <div class="rounded-box bg-base-200 p-3">
                <p class="text-sm font-medium">{spoiler.clone()}</p>
                // Opening the warning is consent for the whole post: text, media and poll
                // arrive together, and asking twice would make one decision cost two clicks.
                <details class="mt-2">
                    <summary
                        class="cursor-pointer text-xs opacity-60"
                        on:click=move |_| set_revealed.set(true)
                    >
                        "Show content"
                    </summary>
                    <p class="mt-2 whitespace-pre-line text-sm leading-relaxed">
                        <span>{status_text(&warned_content, warned_source.as_deref(), warned_format.as_deref(), &warned_emojis)}</span>
                    </p>
                </details>
            </div>
        </Show>
        <Show when=move || !warned>
            <p class="whitespace-pre-line text-sm leading-relaxed">
                <span>{status_text(&plain_content, plain_source.as_deref(), plain_format.as_deref(), &plain_emojis)}</span>
            </p>
        </Show>

        <StatusChips
            tags=status.tags.clone()
            mentions=status.mentions.clone()
            url=status.url.clone()
        />

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
                        <span class="block text-xs opacity-70">{veil_label.clone()}</span>
                    </span>
                </button>
            </div>
        </Show>

        <Show when=move || has_media && !hidden()>
            {grid(items, false, interactive, on_open_media)}
        </Show>

        {poll.map(|poll| PollBlock(PollBlockProps { poll, on_vote }))}
    }
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

/// The author's HTML, with the classes that give it links and paragraph rhythm.
fn status_text(
    content: &str,
    source: Option<&str>,
    format: Option<&str>,
    emojis: &[nodus_client_core::model::CustomEmoji],
) -> AnyView {
    let prepared = if format == Some("mfm") {
        mfm::render(source.unwrap_or(content), emojis)
    } else {
        with_emojis(content, emojis)
    };
    view! {
        <div
            class="break-words [overflow-wrap:anywhere] [&_a]:link [&_a]:link-primary [&_a]:underline-offset-2 [&_p+p]:mt-2"
            inner_html=prepared
        ></div>
    }
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
