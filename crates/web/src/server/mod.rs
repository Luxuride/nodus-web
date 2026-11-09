//! The SSR host: a cookie session in, HTML out.
//!
//! Four pieces, each with one job: [`session`] remembers who the cookie belongs to,
//! [`mastodon`] reads the instance, [`html`] sanitizes what the instance wrote, and
//! [`render`] paints the page. `src/bin/nodus-web-server.rs` wires them into a route table.
//!
//! Everything here compiles only for `ssr`: it speaks axum and the SDK's native transport.

pub mod html;
pub mod mastodon;
pub mod render;
pub mod session;
