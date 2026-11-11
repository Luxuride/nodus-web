//! Splitting a long body into the posts of a thread.
//!
//! The split is at whitespace, because a thread whose parts end mid-word is harder to read
//! than one whose parts are a little short. A single token longer than the limit is cut
//! into hard chunks; there is no other place left to break it.

/// The number of characters a body counts as.
pub fn count(text: &str) -> usize {
    text.chars().count()
}

/// How many posts `text` needs at `limit` characters each.
pub fn posts(text: &str, limit: usize) -> usize {
    if limit == 0 {
        return 1;
    }
    count(text).div_ceil(limit).max(1)
}

/// `text` cut into parts of at most `limit` characters, in reading order.
pub fn split(text: &str, limit: usize) -> Vec<String> {
    if limit == 0 || count(text) <= limit {
        return vec![text.trim().to_owned()];
    }
    let mut parts = Vec::new();
    let mut current = String::new();
    let flush = |current: &mut String, parts: &mut Vec<String>| {
        let part = current.trim_end();
        if !part.is_empty() {
            parts.push(part.to_owned());
        }
        current.clear();
    };

    for token in text.split_inclusive(char::is_whitespace) {
        let len = count(token);
        if len > limit {
            flush(&mut current, &mut parts);
            let mut chunk = String::new();
            for character in token.chars() {
                if count(&chunk) == limit {
                    flush(&mut chunk, &mut parts);
                }
                chunk.push(character);
            }
            current = chunk;
            continue;
        }
        if count(&current) + len > limit {
            if let Some(end) = sentence_break(&current) {
                let tail = current[end..].trim_start().to_owned();
                current.truncate(end);
                flush(&mut current, &mut parts);
                current = tail;
                if count(&current) + len > limit {
                    flush(&mut current, &mut parts);
                }
            } else {
                flush(&mut current, &mut parts);
            }
            current.push_str(token.trim_start());
        } else {
            current.push_str(token);
        }
    }
    flush(&mut current, &mut parts);
    if parts.is_empty() {
        parts.push(String::new());
    }
    parts
}

/// The last sentence end in `current`, when it leaves a useful head behind.
fn sentence_break(current: &str) -> Option<usize> {
    for (index, character) in current.char_indices().rev() {
        if !matches!(character, '.' | '!' | '?' | '\n') {
            continue;
        }
        let end = index + character.len_utf8();
        let after = current[end..].chars().next();
        if after.is_none_or(char::is_whitespace) && count(&current[..end]) >= 12 {
            return Some(end);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{count, posts, split};

    #[test]
    fn a_short_body_is_one_post() {
        assert_eq!(split("hello", 10), vec!["hello"]);
        assert_eq!(posts("hello", 10), 1);
    }

    #[test]
    fn parts_stay_inside_the_limit_and_keep_order() {
        let body = "one two three four five six seven";
        let parts = split(body, 13);
        assert!(parts.iter().all(|part| count(part) <= 13));
        assert_eq!(parts.join(" "), body);
        assert!(parts.len() > 1);
    }

    #[test]
    fn a_break_prefers_the_end_of_a_sentence() {
        let body = "First sentence is here. Second sentence follows it. Third is last.";
        let parts = split(body, 40);
        assert!(parts.iter().all(|part| count(part) <= 40));
        assert!(parts[0].ends_with('.'));
        assert_eq!(parts.join(" "), body);
    }

    #[test]
    fn a_word_longer_than_the_limit_is_cut() {
        let parts = split("abcdefghij", 4);
        assert_eq!(parts, vec!["abcd", "efgh", "ij"]);
    }

    #[test]
    fn paragraphs_survive_as_breaks() {
        let parts = split("first\n\nsecond", 8);
        assert_eq!(parts.join(""), "first\n\nsecond".replace('\n', ""));
    }

    #[test]
    fn unicode_is_counted_in_characters() {
        assert_eq!(count("🦀🦀"), 2);
        assert_eq!(posts("🦀🦀🦀", 2), 2);
    }
}
