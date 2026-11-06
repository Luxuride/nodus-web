# 03 — Data layer

Status: draft · Last revised: 2026-11-06

The data layer is **in the SDK**, not in this repository. `sdk` defines the state
and the four ports it runs on; `web` supplies the browser implementations and
renders the result.

```text
web  ──renders──▶  sdk::AppState  ◀──mutates──  web dispatches sdk::Command
                        │
                        ├── InstanceApi   (impl in web: gloo-net + WebSocket)
                        ├── Discovery     (impl in web: gloo-net to our API)
                        ├── Persistence   (impl in web: IndexedDB)
                        └── Clock         (impl in web: system clock)
```

Consequences for this repository, stated once:

- there is no `src/api/`, `src/masto/`, `src/cache/` or `src/streaming/` here;
- `src/state/` contains one file, `bridge.rs`, which adapts SDK handles to Leptos
  signals and dispatches commands;
- views never call an HTTP endpoint directly — on either plane.

## `nodus-instance` — the instance surface

The SDK does not ship a concrete HTTP client in M0; it ships the surface one must
satisfy:

```rust
pub trait InstanceApi {
    type Error: Debug;
    fn verify_credentials(&self) -> impl Future<Output = Result<Account, Self::Error>>;
    fn timeline(&self, kind: StreamKind, p: PageParams) -> impl Future<Output = Result<Page<Status>, Self::Error>>;
    fn tag_timeline(&self, tag: &str, p: PageParams)     -> impl Future<Output = Result<Page<Status>, Self::Error>>;
    fn status(&self, id: &str)                          -> impl Future<Output = Result<Status, Self::Error>>;
    fn status_context(&self, id: &str)                  -> impl Future<Output = Result<Context, Self::Error>>;
    fn notifications(&self, p: PageParams)              -> impl Future<Output = Result<Page<Notification>, Self::Error>>;
    fn account(&self, id: &LocalId)                     -> impl Future<Output = Result<Account, Self::Error>>;
    fn account_statuses(&self, id: &LocalId, p: PageParams) -> impl Future<Output = Result<Page<Status>, Self::Error>>;
    fn relationship(&self, id: &LocalId)                -> impl Future<Output = Result<Relationship, Self::Error>>;
    fn follow(&self, id: &LocalId)                      -> impl Future<Output = Result<Relationship, Self::Error>>;
    fn unfollow(&self, id: &LocalId)                    -> impl Future<Output = Result<Relationship, Self::Error>>;
    fn post_status(&self, draft: &Draft)                -> impl Future<Output = Result<Status, Self::Error>>;
    fn delete_status(&self, id: &str)                   -> impl Future<Output = Result<(), Self::Error>>;
}
```

Returning `impl Future` instead of `async fn` keeps the futures nameable, which is
what lets the browser implementation (`gloo-net`) and the native one (`reqwest`,
used by the sync worker) satisfy the same trait without `async-trait` boxing.
A companion `Streaming { type Error; fn subscribe(&self, kind: StreamKind) }`
covers the WebSocket side.

Pagination is `PageParams { max_id, since_id, limit }` → `Page<T> { items,
next_max_id }`; cursor-only, because Mastodon has no totals. Feature flags in the
client pick the transport: `wasm` (`gloo-net` + `web-sys::WebSocket`) or `native`
(`reqwest`, used by tests and tooling).

Tokens are `SecretString` at rest in the browser: the type cannot be
`Debug`-printed, and the only place it is interpolated into a URL is the streaming
endpoint, which requires it in the query string.

## Sign-in: PKCE in the browser

```text
1. instance picker           → GET https://host/api/v2/instance   (name, version, capabilities)
2. register the app once     → POST https://host/api/v1/apps
                              {client_name: "Nodus", redirect_uris: <client>/callback,
                               scopes: "read write follow", website}
                              → client_id  (no client_secret: public client)
3. authorize                 → https://host/oauth/authorize?response_type=code&client_id=…
                                 &redirect_uri=…&scope=…&code_challenge=S256(verifier)
                                 &code_challenge_method=S256&state=…
4. code → token              → POST https://host/oauth/token  (grant_type=authorization_code,
                                 code, client_id, redirect_uri, code_verifier)
                              → access_token, created_at, scope
5. verify + persist          → verify_credentials → IndexedDB {host, client_id, access_token,
                                scope, account summary, created_at}
```

Details that matter:

- The `client_id` per host is cached, so signing in again does not re-register.
- `state` and `code_verifier` live in `sessionStorage` for the duration of the
  redirect, keyed by host, and are deleted on return (single-use).
- Scopes requested are the ones the client actually uses. `write:follows` is
  included because following is a first-class client action; our server never
  receives that scope unless the user grants background sync.
- A token created with `read write follow` is a normal Mastodon session; the
  instance shows it under *Preferences → Authorized applications*, which is the
  revocation path we point users to in Settings.
- Sign-out: close the WebSocket, `POST https://host/oauth/revoke` with
  `client_id` + `token`, clear IndexedDB and `sessionStorage`, then (if sync was
  granted) `DELETE /api/v1/sync/token` on our API.

No token is ever sent to our API except on `PUT /api/v1/sync/token`, where it is
the subject of an explicit consent screen
([08-privacy.md](../../recommender/docs/08-privacy.md)).

## Our API client (`Discovery` port)

The SDK declares the second plane as its own trait, so an outage there cannot be
confused with a timeline outage:

```rust
pub trait Discovery {
    type Error: Debug;
    fn meta(&self)                     -> impl Future<Output = Result<Meta, Self::Error>>;
    fn feed(&self, cursor: Option<&str>)-> impl Future<Output = Result<AccountFeed, Self::Error>>;
    fn refresh(&self, feed_id: &str)   -> impl Future<Output = Result<(), Self::Error>>;
    fn send_events(&self, batch: &EventBatch) -> impl Future<Output = Result<(), Self::Error>>;
    fn set_sync(&self, enable: bool, telemetry_opt_in: bool) -> impl Future<Output = Result<(), Self::Error>>;
}
```

`SdkError::is_discovery_only()` exists so the UI can say "timelines are fine; the
Feed tab is degraded" instead of blanking the app.

Repository-side details (in the web implementation of this port):

- Base URL from `PUBLIC_API_BASE` (build-time `env!`/`option_env!`, overridable in
  development), one `gloo-net` wrapper.
- Attaches the Mastodon token as a bearer credential and `X-Contract-Version`
  from `nodus_shared::CONTRACT_VERSION`.
- On boot, `GET /api/v1/meta` is fetched once; the SDK caches it under
  `CacheKey::DiscoveryMeta` and the result gates features: an unknown `features`
  entry is not used, and `client_too_old` (426) short-circuits to an update screen
  instead of a broken feed.
- Error mapping: `ApiErrorBody` → typed `ApiError` → a toast or an inline card,
  never a panic and never a silent empty list.

## Cache (`Persistence` port, IndexedDB in the browser)

The SDK decides *what* to cache and for how long; the web crate only implements
bytes in, bytes out, keyed by `CacheKey`
(`Timeline{host,which,tag}`, `Statuses{host}`, `Notifications{host}`,
`Accounts{host}`, `Emoji{host}`, `Drafts`, `DiscoveryMeta`). Swapping in a
`MemoryPersistence` is how the SDK tests itself, and it is also the fallback when
storage is unavailable.

| Data | Store shape | TTL | Invalidation |
| --- | --- | --- | --- |
| Token + instance + client_id + account summary | one record | none | sign-out |
| Timeline pages | per `(instance, kind, params)`, max 5 pages | 5 min soft | streaming prepend; pull-to-refresh |
| Statuses by id | LRU, 500 entries | 24 h | edit/delete stream events |
| Notifications | max 3 pages | 1 min | streaming |
| Custom emoji | per instance | 24 h | manual refresh |
| Accounts | LRU, 500 entries | 1 h | follow/following change |
| Drafts | per compose target | none | explicit discard or successful post |
| Feed snapshot | **not cached** (server is the source of truth) | — | refetch on mount |

Rules:

- A cache schema version is stored with every record set; an upgrade drops
  incompatible stores instead of crashing on them.
- `navigator.storage.persist()` is requested once; if denied, the app runs
  memory-only and says so in Settings rather than silently losing drafts.
- Writes are batched (`transaction`) so a crash mid-update cannot leave a
  timeline pointing at half-written pages.
- The cache never stores a response that contained a token or a private note;
  only what the UI needs to render.

## State (`AppState`, bridged to signals)

The state tree lives in the SDK and is exposed to views through the bridge:

| SDK handle | Type | Notes |
| --- | --- | --- |
| `AppState::session` | `Option<Session>` | host, account id, acct, scopes; `Option`, guarded by routes. Never holds the token |
| `AppState::timelines` | `HashMap<TimelineKey, TimelinePage>` | key is `(Home\|Local\|Federated\|Tag(_), Option<tag>)` |
| `AppState::feed` | `FeedState` | items, cursor, `generated_at`, degraded flags, dismissed ids |
| `AppState::stream` | `StreamState` | `Disconnected`/`Connecting`/`Connected`/`Reconnecting{attempt}`/`Stopped` |

Web-only UI state (toasts, theme, modal stack) stays in Leptos signals — the SDK
has no opinion about anything that is not Mastodon data.

`state/bridge.rs` subscribes the four handles to one revision signal, so a view
reads `revision()` and then `with_sdk(|sdk| sdk.state().feed.borrow().visible())`.
That is the whole adapter; it is why the SDK can be tested on native.

Stale-while-revalidate is the default: mount renders cached data immediately,
then revalidates, and only shows a skeleton when there is nothing cached.

## Optimistic updates

Optimistic behaviour is expressed as SDK commands, not as component code, so the
same transition is testable without a browser.

**Optimistic, with rollback:** follow/unfollow, boost/unboost, favourite,
bookmark, delete status, mark notifications read.

**Never optimistic:** posting (a failed post must never look sent), media upload,
account migration, moderation reports.

Rollback is a captured inverse closure, not a re-fetch: re-fetching on failure
would also discard any stream events that arrived in between, which is a subtle
bug users notice as "my timeline jumped".

## Offline

- The service worker serves the app shell and hashed assets from `Cache-Control:
  immutable` + a precache manifest; a new version activates on next navigation
  (no surprise reloads while composing).
- Cached timelines and the last feed render offline behind a "showing cached
  content" badge; interactions queue in a small outbox (follow, boost, favourite,
  not-interested) and replay on reconnect in original order, idempotently.
- Compose offline is allowed for drafts only: sending requires the instance.
