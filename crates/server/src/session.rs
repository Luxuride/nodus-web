//! Cookie sessions, and what they are allowed to hold.
//!
//! The cookie is an opaque id. It never carries the OAuth token, and no response body ever
//! contains one: the token lives in this process (and in the file named by
//! `NODUS_SESSIONS`) mapped from the id. A cookie that leaked would let its holder read
//! pages *as* the account, which is already what a browser session means; a token that
//! leaked would also let its holder write to the instance from anywhere, forever.
//!
//! Storage is one JSON file rewritten on every change. That is deliberately the smallest
//! thing that survives a restart: at M2 this becomes the `oauth_token` table described in
//! `nodus-backend/docs/06-ssr.md`, and the endpoint shapes do not change.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard, PoisonError};

use serde::{Deserialize, Serialize};

/// The cookie name. `HttpOnly` because no script needs it, `SameSite=Lax` because a
/// cross-site `POST` must not be able to act as the reader, `Secure` because the SSR host
/// is only ever served over TLS.
pub const COOKIE: &str = "nodus_session";

/// One signed-in visitor: which instance, which account, and the token that proves it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub host: String,
    pub token: String,
    pub account_id: String,
    pub acct: String,
    /// Whether this account answered the moderation queue when it signed in.
    ///
    /// Probed once, at sign-in, rather than on every render: the rail has to know before
    /// the first paint whether to draw the moderation doors, and an instance call per page
    /// would be paid by every reader. A role granted later shows up on the next sign-in.
    #[serde(default)]
    pub admin: bool,
}

/// What the browser is told about its own session. Note the absence of the token.
#[derive(Debug, Clone, Serialize)]
pub struct SessionView<'a> {
    pub host: &'a str,
    pub account_id: &'a str,
    pub acct: &'a str,
}

impl Session {
    #[must_use]
    pub fn view(&self) -> SessionView<'_> {
        SessionView {
            host: &self.host,
            account_id: &self.account_id,
            acct: &self.acct,
        }
    }
}

/// The session store: an id map, plus the file it is mirrored to.
#[derive(Debug)]
pub struct Sessions {
    path: Option<PathBuf>,
    inner: Mutex<HashMap<String, Session>>,
}

impl Sessions {
    /// Load the file if it exists. A corrupt file is reported and then ignored, because a
    /// server that refuses to start over a stale JSON blob is worse than one that asks
    /// everyone to sign in again.
    #[must_use]
    pub fn load(path: Option<PathBuf>) -> Self {
        let inner = path
            .as_ref()
            .and_then(|path| std::fs::read(path).ok())
            .and_then(
                |bytes| match serde_json::from_slice::<HashMap<String, Session>>(&bytes) {
                    Ok(map) => Some(map),
                    Err(error) => {
                        tracing::warn!(%error, "ignoring an unreadable session file");
                        None
                    }
                },
            )
            .unwrap_or_default();
        Self {
            path,
            inner: Mutex::new(inner),
        }
    }

    fn guard(&self) -> MutexGuard<'_, HashMap<String, Session>> {
        // A poisoned map is still readable data: the panic that poisoned it already
        // happened, and refusing every later request helps nobody.
        self.inner.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Stores a proven token and answers the id to put in the cookie.
    pub fn insert(&self, session: Session) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        self.guard().insert(id.clone(), session);
        self.persist();
        id
    }

    #[must_use]
    pub fn get(&self, id: &str) -> Option<Session> {
        self.guard().get(id).cloned()
    }

    pub fn remove(&self, id: &str) {
        self.guard().remove(id);
        self.persist();
    }

    fn persist(&self) {
        let Some(path) = self.path.as_ref() else {
            return;
        };
        let snapshot = serde_json::to_vec_pretty(&*self.guard());
        match snapshot {
            Ok(bytes) => {
                if let Err(error) = std::fs::write(path, bytes) {
                    tracing::error!(%error, path = %path.display(), "could not write sessions");
                }
            }
            Err(error) => tracing::error!(%error, "could not serialise sessions"),
        }
    }
}
