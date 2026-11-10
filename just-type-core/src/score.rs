//! Responisble for tracking the [`Score`] and [`Records`].

use std::time::{Duration, Instant};

use crate::words;

const TEXT_BUFFER: usize = 200;

#[derive(Clone, Copy)]
pub struct Score {
    pub raw_wpm: f64,
    pub net_wpm: f64,
    pub accuracy: f64,
    /// Length of the round the score was measured over.
    pub duration: Duration,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum InputResult {
    Correct,
    Wrong,
}

impl From<bool> for InputResult {
    fn from(correct: bool) -> Self {
        if correct { Self::Correct } else { Self::Wrong }
    }
}

pub struct Round {
    text: Vec<char>,
    results: Vec<InputResult>,
    start_time: Option<Instant>,
    /// Copied from the settings when the round starts, so changing them can't affect it.
    pub duration: Duration,
    keystrokes: usize,
    correct_keystrokes: usize,
}

impl Round {
    pub fn new(duration: Duration) -> Self {
        let mut text = vec![];
        words::extend_text(&mut text, TEXT_BUFFER);

        Self {
            text,
            results: vec![],
            start_time: None,
            duration,
            keystrokes: 0,
            correct_keystrokes: 0,
        }
    }

    pub fn text(&self) -> &[char] {
        &self.text
    }

    pub fn results(&self) -> &[InputResult] {
        &self.results
    }

    pub fn started(&self) -> bool {
        self.start_time.is_some()
    }

    pub fn type_char(&mut self, c: char) {
        _ = self.start_time.get_or_insert_with(Instant::now);

        // Prevent input after time has ended.
        // Because the round may end after time runs out
        // it would still be possible to give input although the
        // remaining time is 0. This guard prevents that.
        if self.remaining().is_zero() {
            return;
        }

        if let Some(&expected) = self.text.get(self.results.len()) {
            let correct = c == expected;
            self.keystrokes += 1;
            self.correct_keystrokes += usize::from(correct);
            self.results.push(correct.into());
        }

        words::extend_text(&mut self.text, self.results.len() + TEXT_BUFFER);
    }

    /// Deletes the last typed character.
    /// If `word` is true, it deletes everything back to the previous space.
    pub fn backspace(&mut self, word: bool) {
        // Prevent backspace after time has ended.
        // Because the round may end after time runs out
        // it would still be possible to give input although the
        // remaining time is 0. This guard prevents that.
        if self.remaining().is_zero() {
            return;
        }

        self.results.pop();
        if word {
            while !self.results.is_empty() && self.text[self.results.len() - 1] != ' ' {
                self.results.pop();
            }
        }
    }

    /// The full duration until the first keystroke, then counts down to zero.
    pub fn remaining(&self) -> Duration {
        self.start_time
            .map_or(self.duration, |start| self.duration.saturating_sub(start.elapsed()))
    }

    pub fn score(&self) -> Score {
        let correct_chars = self.results.iter().filter(|&&r| r == InputResult::Correct).count();

        compute_score(self.keystrokes, self.correct_keystrokes, correct_chars, self.duration)
    }
}

/// How a [`Score`] compares with earlier rounds of the same test.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Records {
    /// The best wpm so far, this round included.
    pub personal_best: f64,
    /// Whether this round beat the previous best.
    pub new_personal_best: bool,
    /// Average wpm of the last `recent_count` rounds, this one included.
    pub recent_average: f64,
    pub recent_count: usize,
}

fn wpm(chars: usize, elapsed: Duration) -> f64 {
    let minutes = elapsed.as_secs_f64() / 60.0;
    if minutes == 0.0 {
        return 0.0;
    }
    (chars as f64 / 5.0) / minutes
}

/// `keystrokes` and `correct_keystrokes` count every key press, including ones later erased with backspace,
/// `correct_chars` counts only correct chars left in the text.
fn compute_score(keystrokes: usize, correct_keystrokes: usize, correct_chars: usize, elapsed: Duration) -> Score {
    let raw_wpm = wpm(keystrokes, elapsed);
    let net_wpm = wpm(correct_chars, elapsed);
    let accuracy = if keystrokes > 0 {
        correct_keystrokes as f64 / keystrokes as f64 * 100.0
    } else {
        100.0
    };

    Score {
        raw_wpm,
        net_wpm,
        accuracy,
        duration: elapsed,
    }
}
