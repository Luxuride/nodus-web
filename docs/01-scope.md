# 01 — Scope

Status: draft · Last revised: 2026-11-06 (decisions D5, D6)

A **complete Mastodon web client** written in Rust and compiled to WebAssembly,
plus the discovery views that are the reason it exists. "Complete" means a user
can switch to it as their daily client, not that it has a settings toggle for
every instance feature.

## Why Leptos

- **Fine-grained reactivity.** Signals, not VDOM diffing: a 300-post timeline
  re-renders the one post that changed. This is the structural difference that
  matters for dense lists, and the main reason not to use Yew here.
- **One language end to end.** `mastodon-reccom-shared` (DTOs) and
  `mastodon-reccom-mastodon` (instance client) are compiled into the browser, the
  API and the worker, so a wrong field is a compile error on every side at once
  instead of a runtime 500 in production.
- **Hydration available later** without changing the app model — relevant only if
  some public, unauthenticated page ever needs SEO ([05-quality.md](05-quality.md),
  decision D8).
- Cost accepted: compile times, weaker devtools than JS frameworks, a smaller
  contributor pool, and a hard bundle-size budget.

Rejected: Yew (VDOM, weaker list story), Dioxus (multi-platform we do not need),
Svelte/React (would give up shared types — the entire reason this is Rust).

## Parity matrix

Phases are **delivery order**: M2 ships a client worth using, M3 completes
daily-driver parity, M4 adds the long tail.

### M2 — core reading (cannot ship without)

| Feature | Notes |
| --- | --- |
| Sign-in to any instance | dynamic app registration + PKCE in the browser, instance picker with remembered hosts |
| Home timeline | REST page fetch + streaming merge, virtualized |
| Local / Federated timelines | same component, different endpoint |
| Status rendering | content warning, visibility icons, media grid, poll, boost-of-deleted, edited-at, filters |
| Thread view | ancestors + descendants in one column, "load more" at each gap |
| Notifications | paginated, grouped types, mark-as-read, filter by type |
| Profiles | header, fields, tabs (posts, replies, media), follow/unfollow/block/mute, moved-to banner |
| Compose | text, CW, visibility, media upload, poll; local draft persistence |
| **Feed tab** | recommended accounts + posts, reasons, follow in place, dismiss, feedback |
| Settings | theme, instance, sync consent, data export/delete, sign out + revoke |

### M3 — daily-driver parity

| Feature | Notes |
| --- | --- |
| Lists | CRUD + list timelines |
| Bookmarks / favourites | timelines and toggle actions |
| Search | accounts, hashtags, statuses; `/` focus shortcut |
| Hashtag timeline | tag + tag/local |
| Filters | server-side keyword/severity filters applied to rendered content |
| Moderation surface | mute/block lists, domain blocks, report dialog |
| Streaming behaviours | gap detection, reconnect with backoff, "N new posts" banner instead of a jump |
| Media | image lightbox with zoom/pan, video/audio player, alt-text display, ALT badge |
| Custom emoji | shortcode → `<img>` from the instance list, cached per instance |
| Keyboard shortcuts | `j/k/enter/r/b/f/n`, `g`-prefix navigation, `?` help overlay |
| i18n | Fluent via `leptos_i18n`, English first |
| Offline / PWA | installable; cached timelines render offline behind a staleness badge |

### M4 — long tail

Announcements/trends, scheduled posts, follow requests for locked accounts,
featured hashtags, CSV import/export, account migration flow, web push (needs
VAPID per instance), quote posts when the instance advertises them.

### Out of scope

- **Server administration.** This is a user client.
- **Non-Mastodon forks' extensions.** Akkoma/Pleroma/GoToSocial features
  degrade gracefully; they are not targets.
- **Storing passwords.** Credentials only ever touch the instance's own login
  page during the `authorize` redirect.
- **Ranking math.** The client renders ranked items with reasons; it never
  recomputes scores ([02-architecture.md](02-architecture.md)).

## The Feed tab is part of M2

Deliberately, the recommender's surface ships in the *first* usable client
rather than after parity. The reason is sequencing risk: if the Feed arrives only
after months of parity work, we learn nothing about whether recommendations work
until the expensive part is finished. The feed API exists from backend M2, so
the tab has something real to render.

Contract dependency: the Feed tab consumes `GET /api/v1/feed/accounts` and
`/feed/posts` from our API while the rest of the client talks to the instance
directly — two different origins, two different failure modes, one app.
