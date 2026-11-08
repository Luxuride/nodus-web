# 06 — The server-rendered first page

Status: shipped (`crates/server`, `nodus-ssr.service` on LXC 100). Revision 1.

The client is a 2 MB wasm bundle. Downloading it before the first paint is the slowest
thing about a visit, and the browser cannot render a timeline it has no session for. So
the first document is painted by a small Rust server that already holds the session.

## What the server does

| Route | Answer |
| --- | --- |
| `GET /` | landing page (brand, what Nodus is, `Sign in`) or the home timeline |
| `GET /home` | the home timeline, or a landing page with `Sign in to see your timelines.` |
| `POST /api/session` | takes `{"host","token"}`, **verifies** it against the instance, stores it server-side, answers `HttpOnly` cookie `nodus_session` |
| `DELETE /api/session` | forgets the session and clears the cookie |
| `GET /healthz` | `{"status":"ok","script":…,"stylesheet":…}` once the asset directory is readable |

The token is verified before it is stored, so a wrong token is a `401`, not a session that
fails later in a confusing way.

## The cookie is a pointer, never the token

`nodus_session` is `HttpOnly; Secure; SameSite=Lax` and holds an opaque id. The token lives
in `/var/lib/nodus/sessions.json` (`0600`, owned by the `nodus` user). Consequences:

* no rendered document can leak the token, and neither can an XSS payload, because
  JavaScript cannot read the cookie;
* the browser keeps its own copy of the token for the client's API calls and streaming,
  so the app works exactly as before when the server is unreachable;
* deleting a row revokes the server session; signing out also revokes the OAuth token
  through the instance.

## The island: the client does not re-fetch what the server painted

The document carries the statuses it painted as JSON in `#nodus-page`:

```html
<script type="application/json" id="nodus-page">{"which":"home","statuses":[…],"cursor":"…"}</script>
```

`crates/web/src/state/bridge/seed.rs` reads it during `bridge::init()`, before the first
component renders, and calls `Sdk::adopt_page(…)`, which dispatches `StatusesUpserted`,
`AccountsUpserted` and `TimelineLoaded` in one go. Because the store already holds the
page, `timeline::held(Timeline::Home, None)` is `true`, so `Home` renders content and
`ensure` answers without a request: no spinner, no duplicate fetch.

A missing or malformed island is not an error: the client starts cold and fetches the page
itself, which is exactly what the static deployment does.

## Configuration

| Variable | Default |
| --- | --- |
| `NODUS_LISTEN` | `127.0.0.1:8080` |
| `NODUS_ASSET_DIR` | `/srv/nodus` |
| `NODUS_SESSIONS` | `/var/lib/nodus/sessions.json` |

## Build and deploy

The binary links `aws-lc` through `reqwest`, so it must be built against the glibc the
target runs. For Debian 12 (LXC 100):

```sh
docker run --rm \
  -v /var/home/lux/Projects/MastodonReccom:/src:ro \
  -v /var/home/lux/Projects/srv-build:/out \
  -v nodus-cargo-cache:/usr/local/cargo/registry \
  -e CFLAGS=-std=gnu11 -e AWS_LC_SYS_CFLAGS=-std=gnu11 \
  rust:1-bookworm bash -lc 'export PATH=/usr/local/cargo/bin:$PATH && mkdir -p /build && \
    cp -a /src/. /build/ && cd /build/web && cargo build --release -p nodus-server && \
    cp target/release/nodus-server /out'
```

`CFLAGS=-std=gnu11` is not optional on bookworm: gcc 12 compiles `aws-lc` in C23 mode and
the object then references `__isoc23_sscanf`, which glibc 2.36 does not export
(`rust-lld: error: undefined symbol: __isoc23_sscanf`).

Then install the binary, the unit (`nodus-ssr.service`, `ProtectSystem=strict`,
`ReadWritePaths=/var/lib/nodus`) and route the two entry points in `nodus.caddy` **ahead**
of the static fallback:

```caddyfile
handle /api/session { reverse_proxy 127.0.0.1:8080 }  # before handle /api/*
handle /home { reverse_proxy 127.0.0.1:8080 }
handle / { reverse_proxy 127.0.0.1:8080 }
```

`/api/session` must come before `/api/*` or the API proxy claims it. Every other route is
still answered by `root * /srv/nodus` with `try_files {path} /index.html`, so client-side
routes are unchanged.

## The host must be able to reach the instance

The server calls the reader's instance directly (`verify_credentials`, `timelines/home`), so
it needs a route to it. When the SSR host and the instance share a machine behind Caddy,
the public name usually resolves to an address the container cannot reach (hairpin NAT):
the symptom is `POST /api/session` answering `401 token rejected` for a token that works in
the browser, because the verification never left the box. Point the name at loopback so
Caddy answers with the real certificate:

```sh
echo '127.0.0.1 mastodon.luxuride.eu nodus.luxuride.eu' >> /etc/hosts   # inside the SSR host
curl -s -o /dev/null -w '%{http_code}\n' https://mastodon.luxuride.eu/api/v1/instance  # 200
```

## What this does not do

* No SSR for authenticated sub-routes yet (`/notifications`, `/settings/…`): the store
  seeds them on the client, so they paint from cache without a spinner.
* No server-side session renewal or refresh-token storage.
* The share page `/s/<host>/<id>` belongs to the recommender backend, not this server.
