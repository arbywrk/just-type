use std::time::Duration;

pub struct Score {
    pub raw_wpm: f64,
    pub net_wpm: f64,
    pub accuracy: f64,
}

fn wpm(chars: usize, elapsed: Duration) -> f64 {
    let minutes = elapsed.as_secs_f64() / 60.0;
    if minutes == 0.0 {
        return 0.0;
    }
    (chars as f64 / 5.0) / minutes
}

/// `keystrokes` and `correct_keystrokes` count every key press, including ones later
/// erased with backspace; `correct_chars` counts only correct chars left in the text.
pub fn compute_score(
    keystrokes: usize,
    correct_keystrokes: usize,
    correct_chars: usize,
    elapsed: Duration,
) -> Score {
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
    }
}
