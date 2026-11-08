# nodus-server

The server-rendered first page for the Nodus web client.

The browser bundle is the app; this binary exists for the two things a wasm module
cannot do well: paint the first page without downloading 2 MB of wasm first, and hold
the reader's session where JavaScript cannot read it.

* `GET /` and `GET /home` answer with a painted page (timeline statuses included).
* `POST /api/session` takes `{"host","token"}` from the client and answers with an
  opaque, `HttpOnly` cookie; the token itself never appears in a document.
* `GET /healthz` answers `ok` once the asset directory is readable.

The client finishes its own sign-in, posts the session here, and adopts the page this
server painted through the JSON island in the document (`#nodus-page`), so no screen
starts with a spinner.

## Configuration

| Variable | Default | Meaning |
| --- | --- | --- |
| `NODUS_LISTEN` | `127.0.0.1:8080` | Address to bind |
| `NODUS_ASSET_DIR` | `/srv/nodus` | Where `trunk` wrote `index.html` and its hashed assets |
| `NODUS_SESSIONS` | `/var/lib/nodus/sessions.json` | Session store; keep it `0600` |

## Build

```sh
cd web && cargo build --release -p nodus-server     # must match the host's glibc
```

The binary links `aws-lc` through `reqwest`, so build it in an image with the same glibc
generation as the target (a `rust:1-bookworm` image for a Debian 12 host). See
`../docs/06-ssr.md` for the deployment recipe.
