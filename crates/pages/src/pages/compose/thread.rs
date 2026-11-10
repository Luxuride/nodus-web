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
            flush(&mut current, &mut parts);
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
