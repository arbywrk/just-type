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

/// Greedily splits `text` into lines of at most `width` chars, breaking after spaces.
/// Earlier lines never change when words are appended, so lines don't jump while typing.
pub fn wrap_lines(text: &[char], width: usize) -> Vec<Range<usize>> {
    let mut lines = vec![];
    let mut line_start = 0;
    let mut word_start = 0;

    for i in 0..=text.len() {
        let word_end = if i == text.len() {
            i
        } else if text[i] == ' ' {
            i + 1
        } else {
            continue;
        };

        if word_end - line_start > width && word_start > line_start {
            lines.push(line_start..word_start);
            line_start = word_start;
        }
        word_start = word_end;
    }
    if line_start < text.len() {
        lines.push(line_start..text.len());
    }

    lines
}

pub fn visible_lines(
    text: &[char],
    cursor: usize,
    width: usize,
    count: usize,
) -> impl Iterator<Item = Range<usize>> {
    let lines = wrap_lines(text, width);
    let current_line = lines
        .iter()
        .position(|line| line.contains(&cursor))
        .unwrap_or(lines.len().saturating_sub(1));
    // Keep the current line in the middle once past the first, so there's always one line ahead.
    let first_visible = current_line.saturating_sub(1);

    lines.into_iter().skip(first_visible).take(count)
}
