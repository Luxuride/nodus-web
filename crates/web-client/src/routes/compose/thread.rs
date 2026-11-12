//! Turning one draft into the posts of a thread.

use crate::state::bridge::actions::NewStatus;

/// The draft as posts, each within the character limit.
///
/// Attachments travel in order, four to a post at most. The poll belongs to the first post;
/// a poll on every part would ask the same question twice.
#[cfg(test)]
pub(crate) fn statuses(base: NewStatus, limit: usize) -> Vec<NewStatus> {
    statuses_from(vec![base.text.clone()], base, limit)
}

/// The same split, for pages the writer laid out by hand.
///
/// Each page may still be longer than the limit, so every page is split again; the writer's
/// boundaries are kept as the outer order. Empty pages are dropped, except that a thread
/// with no text at all remains one empty draft for the submit guard to reject.
pub(crate) fn statuses_from(texts: Vec<String>, base: NewStatus, limit: usize) -> Vec<NewStatus> {
    let limit = limit.max(1);
    let texts = texts
        .into_iter()
        .flat_map(|text| nodus_pages::pages::compose::thread::split(&text, limit))
        .filter(|text| !text.trim().is_empty())
        .collect::<Vec<_>>();
    if texts.is_empty() {
        let mut only = base;
        only.text.clear();
        return vec![only];
    }
    let parts = texts.len();
    let media = base.media_ids.clone();
    let per = media.len() / parts;
    let extra = media.len() % parts;
    let mut media = media.into_iter();
    let mut out = Vec::with_capacity(parts);
    for (index, text) in texts.into_iter().enumerate() {
        let mut part = base.clone();
        part.text = text;
        let take = per + usize::from(index < extra);
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
    use nodus_client_core::model::Visibility;

    use super::{super::inline::LIMIT, statuses, statuses_from};
    use crate::state::bridge::actions::NewStatus;

    #[test]
    fn a_short_draft_is_one_post() {
        let posts = statuses(NewStatus::text("hello", Visibility::Public), LIMIT);
        assert_eq!(posts.len(), 1);
    }

    #[test]
    fn hand_made_pages_keep_their_order() {
        let posts = statuses_from(
            vec!["first".to_owned(), String::new(), "third".to_owned()],
            NewStatus::text("", Visibility::Public),
            LIMIT,
        );
        assert_eq!(posts.len(), 2);
        assert_eq!(posts[0].text, "first");
        assert_eq!(posts[1].text, "third");
    }

    #[test]
    fn a_long_hand_made_page_is_split_again() {
        let posts = statuses_from(
            vec!["word ".repeat(120), "last".to_owned()],
            NewStatus::text("", Visibility::Public),
            LIMIT,
        );
        assert!(posts.len() > 2);
        assert_eq!(posts.last().unwrap().text, "last");
    }

    #[test]
    fn media_is_spread_evenly_across_hand_made_pages() {
        let mut base = NewStatus::text("", Visibility::Public);
        base.media_ids = vec!["1".into(), "2".into(), "3".into(), "4".into()];
        let posts = statuses_from(
            vec!["one".to_owned(), "two".to_owned(), "three".to_owned()],
            base,
            LIMIT,
        );
        assert_eq!(posts.len(), 3);
        assert_eq!(
            posts
                .iter()
                .map(|post| post.media_ids.len())
                .collect::<Vec<_>>(),
            vec![2, 1, 1]
        );
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
