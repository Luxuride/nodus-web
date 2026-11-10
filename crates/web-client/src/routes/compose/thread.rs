//! Turning one draft into the posts of a thread.

use crate::state::bridge::actions::NewStatus;

/// The draft as posts, each within the character limit.
///
/// Attachments travel in order, four to a post at most. The poll belongs to the first post;
/// a poll on every part would ask the same question twice.
pub(crate) fn statuses(base: NewStatus, limit: usize) -> Vec<NewStatus> {
    let texts = nodus_pages::pages::compose::thread::split(&base.text, limit.max(1));
    let parts = texts.len();
    let media = base.media_ids.clone();
    let per = if parts == 1 {
        media.len()
    } else {
        media.len().div_ceil(parts).clamp(1, 4)
    };
    let mut media = media.into_iter();
    let mut out = Vec::with_capacity(parts);
    for (index, text) in texts.into_iter().enumerate() {
        let mut part = base.clone();
        part.text = text;
        let take = if index + 1 == parts { media.len() } else { per };
        part.media_ids = media.by_ref().take(take).collect();
        if index > 0 {
            part.poll = None;
        }
        out.push(part);
    }
    out
}

#[cfg(test)]
mod tests {
    use nodus_instance::model::Visibility;

    use super::{super::inline::LIMIT, statuses};
    use crate::state::bridge::actions::NewStatus;

    #[test]
    fn a_short_draft_is_one_post() {
        let posts = statuses(NewStatus::text("hello", Visibility::Public), LIMIT);
        assert_eq!(posts.len(), 1);
    }

    #[test]
    fn media_is_spread_across_the_posts() {
        let mut base = NewStatus::text("word ".repeat(300), Visibility::Public);
        base.media_ids = vec!["1".into(), "2".into(), "3".into(), "4".into()];
        let posts = statuses(base, LIMIT);
        assert!(posts.len() > 1);
        assert_eq!(
            posts.iter().map(|post| post.media_ids.len()).sum::<usize>(),
            4
        );
    }
}
