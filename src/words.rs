use std::{ops::Range, sync::LazyLock};

use rand::seq::IndexedRandom;
use serde::Deserialize;

#[derive(Deserialize)]
struct Language {
    words: Vec<String>,
}

static WORDS: LazyLock<Vec<String>> = LazyLock::new(|| {
    let lang: Language = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/resources/languages/english.json"
    )))
    .expect("failed to parse english.json");

    lang.words
});

/// Appends random space-separated words until `text` has at least `min_len` chars.
pub fn extend_text(text: &mut Vec<char>, min_len: usize) {
    let mut rng = rand::rng();

    while text.len() < min_len {
        let word = WORDS.choose(&mut rng).expect("word list is empty");

        if !text.is_empty() {
            text.push(' ');
        }
        text.extend(word.chars());
    }
}

/// Greedily splits `text` into lines no wider than `max_width`, breaking after spaces.
///
/// `word_width` measures a word together with its trailing space, if any.
/// A word wider than `max_width` gets a line of its own.
/// Earlier lines never change when words are appended.
pub fn wrap_lines(text: &[char], max_width: f32, mut word_width: impl FnMut(&[char]) -> f32) -> Vec<Range<usize>> {
    let mut lines = vec![];
    let mut line_start = 0;
    let mut line_width = 0.0;
    let mut word_start = 0;

    for i in 0..=text.len() {
        let word_end = if i == text.len() {
            i
        } else if text[i] == ' ' {
            i + 1
        } else {
            continue;
        };
        if word_end == word_start {
            continue;
        }

        let width = word_width(&text[word_start..word_end]);
        if line_width + width > max_width && word_start > line_start {
            lines.push(line_start..word_start);
            line_start = word_start;
            line_width = 0.0;
        }
        line_width += width;
        word_start = word_end;
    }
    if line_start < text.len() {
        lines.push(line_start..text.len());
    }

    lines
}

/// Returns up to `count` lines around the one containing `cursor`.
pub fn visible_lines(lines: &[Range<usize>], cursor: usize, count: usize) -> &[Range<usize>] {
    let current_line = lines
        .iter()
        .position(|line| line.contains(&cursor))
        .unwrap_or(lines.len().saturating_sub(1));
    // Keep the current line in the middle once past the first, so there's always one line ahead.
    let first_visible = current_line.saturating_sub(1);
    let last_visible = (first_visible + count).min(lines.len());

    &lines[first_visible..last_visible]
}

#[cfg(test)]
mod tests {
    use rstest::{fixture, rstest};

    use super::*;

    mod helpers {
        pub fn chars(s: &str) -> Vec<char> {
            s.chars().collect()
        }

        /// Measures every char as one unit wide.
        pub fn unit_width(word: &[char]) -> f32 {
            word.len() as f32
        }
    }

    mod wrap_lines {
        use super::*;

        #[rstest]
        #[case::wraps_after_spaces_within_width("aa bb cc dd", 6.0, vec![0..6, 6..11])]
        #[case::overlong_word_gets_its_own_line("a bbbbbbbb c", 4.0, vec![0..2, 2..11, 11..12])]
        fn wraps(#[case] text: &str, #[case] max_width: f32, #[case] expected: Vec<Range<usize>>) {
            let text = helpers::chars(text);

            assert_eq!(wrap_lines(&text, max_width, helpers::unit_width), expected);
        }

        #[rstest]
        fn appending_words_keeps_earlier_lines() {
            let short = wrap_lines(&helpers::chars("aa bb cc"), 6.0, helpers::unit_width);
            let long = wrap_lines(&helpers::chars("aa bb cc dd ee"), 6.0, helpers::unit_width);

            assert_eq!(long[..short.len() - 1], short[..short.len() - 1]);
        }
    }

    mod visible_lines {
        use super::*;

        #[fixture]
        fn lines() -> Vec<Range<usize>> {
            vec![0..3, 3..6, 6..9, 9..12]
        }

        #[rstest]
        #[case::starts_at_the_top_on_the_first_line(1, vec![0..3, 3..6, 6..9])]
        #[case::keeps_cursor_line_second_once_past_the_first(7, vec![3..6, 6..9, 9..12])]
        #[case::shows_fewer_lines_at_the_end(10, vec![6..9, 9..12])]
        fn shows(lines: Vec<Range<usize>>, #[case] cursor: usize, #[case] expected: Vec<Range<usize>>) {
            assert_eq!(visible_lines(&lines, cursor, 3), expected);
        }
    }
}
