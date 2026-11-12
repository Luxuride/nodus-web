//! Account preferences and instance announcements.

use super::instance_transport;
use nodus_client_core::actions::SettingsActions;
use nodus_client_core::model::{Announcement, Preferences};

/// The account's posting and reading defaults.
pub async fn preferences() -> Result<Preferences, String> {
    let transport = instance_transport().ok_or_else(|| "Not signed in.".to_owned())?;
    transport
        .preferences()
        .await
        .map_err(|error| error.to_string())
}

/// Announcements the instance is showing.
pub async fn announcements() -> Result<Vec<Announcement>, String> {
    let transport = instance_transport().ok_or_else(|| "Not signed in.".to_owned())?;
    transport
        .announcements()
        .await
        .map_err(|error| error.to_string())
}

/// Marks an announcement as read so it stops appearing.
pub async fn dismiss_announcement(id: &str) -> Result<(), String> {
    let transport = instance_transport().ok_or_else(|| "Not signed in.".to_owned())?;
    transport
        .dismiss_announcement(id)
        .await
        .map_err(|error| error.to_string())
}
