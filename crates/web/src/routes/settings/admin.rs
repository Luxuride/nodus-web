//! The admin-scope grant, shared by the moderation and administration gates.
//!
//! This is not a settings section: moderation is a place, reached from the rail, and the
//! permission it needs is asked for at the door rather than at sign-in.

use crate::state::bridge::admin_host;
use crate::state::oauth;

/// Starts the authorization that gives this browser an admin-scoped token.
///
/// The moderation screens call this from their own gate: asking for `admin:read` at sign-in
/// would hand every reader far more access than a reader needs, so the scope is requested
/// the moment it is actually needed and can be forgotten again afterwards.
pub async fn request_grant() -> Result<(), String> {
    let host = admin_host().ok_or("There is no instance to ask yet.")?;
    oauth::begin_admin(&host).await
}
