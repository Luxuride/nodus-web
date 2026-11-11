//! The thread pages under the main editor.
//!
//! The writer can add a page, edit it, split it in two, throw it away, or seed the pages from
//! a long first draft. Every page is a real editor in the order it will be posted; a page
//! that is still too long is split again by the publish path.

use std::sync::atomic::{AtomicU64, Ordering};

use leptos::prelude::*;
use nodus_pages::pages::compose::thread::{count, split};

static NEXT: AtomicU64 = AtomicU64::new(1);

#[derive(Clone)]
pub(crate) struct Page {
    id: u64,
    ordinal: RwSignal<usize>,
    pub(crate) text: RwSignal<String>,
}

impl Page {
    fn new(text: String, ordinal: usize) -> Self {
        Self {
            id: NEXT.fetch_add(1, Ordering::Relaxed),
            ordinal: RwSignal::new(ordinal),
            text: RwSignal::new(text),
        }
    }
}

fn renumber(pages: &RwSignal<Vec<Page>>) {
    pages.update(|pages| {
        for (index, page) in pages.iter().enumerate() {
            page.ordinal.set(index + 1);
        }
    });
}

/// Split one page near its middle at a word boundary and leave the tail as the next page.
fn split_page(page: Page, pages: RwSignal<Vec<Page>>) {
    let text = page.text.get_untracked();
    let Some((head, tail)) = half(&text) else {
        return;
    };
    page.text.set(head);
    let mut inserted = false;
    pages.update(|pages| {
        if let Some(index) = pages.iter().position(|candidate| candidate.id == page.id) {
            pages.insert(index + 1, Page::new(tail, index + 2));
            inserted = true;
        }
    });
    if inserted {
        renumber(&pages);
    }
}

fn half(text: &str) -> Option<(String, String)> {
    let total = count(text);
    if total < 3 {
        return None;
    }
    let middle = total / 2;
    let mut best = None;
    for (index, character) in text.char_indices() {
        if !character.is_whitespace() {
            continue;
        }
        let distance = count(&text[..index]).abs_diff(middle);
        if best.is_none_or(|(_, best_distance)| distance < best_distance) {
            best = Some((index, distance));
        }
    }
    let (index, _) = best?;
    let head = text[..index].trim_end();
    let tail = text[index..].trim_start();
    (!head.is_empty() && !tail.is_empty()).then(|| (head.to_owned(), tail.to_owned()))
}

/// The editable pages after the first one, plus the buttons that make and reshape them.
#[component]
pub(crate) fn ThreadPages(
    text: ReadSignal<String>,
    pages: RwSignal<Vec<Page>>,
    set_error: WriteSignal<Option<String>>,
    limit: Signal<usize>,
) -> impl IntoView {
    let add = Callback::new(move |()| {
        pages.update(|pages| pages.push(Page::new(String::new(), pages.len() + 1)));
        set_error.set(None);
    });

    // The first page is the draft the writer already has. Seeding once on mount is what
    // makes the visual editor open with the whole draft in it, instead of an empty list
    // beside a hidden main editor.
    let seeded = StoredValue::new(false);
    // The page signals have to outlive the effect that builds them: an effect rerun disposes
    // the owner it created them under, and the list would then hold disposed signals.
    let owner = Owner::current();
    Effect::new(move |_| {
        if seeded.get_value() || !pages.with(Vec::is_empty) {
            return;
        }
        seeded.set_value(true);
        let body = text.get();
        let mut pieces = split(&body, limit.get().max(1));
        if pieces.is_empty() {
            pieces.push(String::new());
        }
        let built: Vec<Page> = owner
            .as_ref()
            .map(|owner| {
                owner.with(|| {
                    pieces
                        .into_iter()
                        .enumerate()
                        .map(|(index, text)| Page::new(text, index + 1))
                        .collect()
                })
            })
            .unwrap_or_default();
        pages.set(built);
        set_error.set(None);
    });

    view! {
        <section class="flex flex-col gap-2 rounded-box border border-base-300 bg-base-100 p-3">
            <div class="flex flex-wrap items-center gap-2">
                <span class="text-xs font-semibold tracking-wide uppercase opacity-50">"Thread"</span>
                <span class="text-xs opacity-60">
                    {move || format!("{} posts", pages.with(|pages| pages.len()))}
                </span>
                <button type="button" class="btn btn-ghost btn-xs" on:click=move |_| add.run(())>
                    "+ Add post"
                </button>
            </div>

            <Show when=move || !pages.with(Vec::is_empty)>
                <div class="flex flex-col gap-2">
                    <For each=move || pages.get() key=|page| page.id let:page>
                        <PageCard page=page pages=pages limit=limit/>
                    </For>
                    <p class="text-xs opacity-60">
                        "Attachments are spread across the thread. Every page has to fit the server's "
                        {move || limit.get()}
                        "-character limit before you post."
                    </p>
                </div>
            </Show>
        </section>
    }
}

/// One editable page. The page owns its signal, so typing does not rebuild the list.
#[component]
fn PageCard(page: Page, pages: RwSignal<Vec<Page>>, limit: Signal<usize>) -> impl IntoView {
    let text = page.text;
    let id = page.id;
    let ordinal = page.ordinal;
    let remove = move |_| {
        pages.update(|pages| pages.retain(|page| page.id != id));
        renumber(&pages);
    };
    let split = move |_| split_page(page.clone(), pages);

    view! {
        <div class="rounded-box border border-base-300 bg-base-100 p-2">
            <div class="mb-1 flex items-center justify-between gap-2">
                <span class="text-xs font-medium">{move || format!("Post {}", ordinal.get())}</span>
                <div class="flex items-center gap-1">
                    <span class="text-xs tabular-nums" class:text-warning=move || { count(&text.get()) > limit.get() }>
                        {move || format!("{}/{}", count(&text.get()), limit.get())}
                    </span>
                    <button
                        type="button"
                        class="btn btn-ghost btn-xs"
                        title="Split this post in two"
                        aria-label="Split this post in two"
                        on:click=split
                    >
                        "Split"
                    </button>
                    <button
                        type="button"
                        class="btn btn-ghost btn-xs text-error"
                        title="Remove this post"
                        aria-label="Remove this post"
                        on:click=remove
                    >
                        "Remove"
                    </button>
                </div>
            </div>
            <textarea
                class="textarea textarea-bordered min-h-24 w-full resize-y text-sm leading-relaxed"
                placeholder="Continue the thread…"
                prop:value=move || text.get()
                on:input=move |event| text.set(event_target_value(&event))
            ></textarea>
        </div>
    }
}

/// The thread as the writer laid it out.
pub(crate) fn texts(pages: &[Page]) -> Vec<String> {
    pages.iter().map(|page| page.text.get_untracked()).collect()
}
