//! The instance's own limits, remembered after the first answer.

use std::cell::Cell;

thread_local! {
    static MAX_CHARACTERS: Cell<Option<usize>> = const { Cell::new(None) };
}

/// The longest body this instance accepts, or Mastodon's default until it answers.
pub async fn max_characters() -> usize {
    if let Some(limit) = MAX_CHARACTERS.with(Cell::get) {
        return limit;
    }
    let limit = super::actions::instance_info()
        .await
        .map(|instance| instance.max_characters())
        .unwrap_or(500);
    MAX_CHARACTERS.with(|slot| slot.set(Some(limit)));
    limit
}
