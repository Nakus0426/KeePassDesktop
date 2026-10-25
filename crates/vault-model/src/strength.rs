//! Password strength.
//!
//! Heuristic scoring, matching the reference client's model so the meter reads
//! the same in both apps: a length base plus charset bonuses minus penalties for
//! common words, sequences and repeats. It is a UI hint, not a security claim.

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Score {
    VeryWeak = 0,
    Weak = 1,
    Fair = 2,
    Strong = 3,
    VeryStrong = 4,
}

impl Score {
    pub const fn dots(self) -> u8 {
        self as u8 + 1
    }
}

/// Score a candidate password.
pub fn score(password: &str) -> Score {
    let len = password.chars().count();
    if len == 0 {
        return Score::VeryWeak;
    }

    let mut points: i32 = match len {
        0..=7 => 0,
        8..=11 => 10,
        12..=15 => 20,
        16..=19 => 28,
        _ => 34,
    };

    let has_lower = password.chars().any(|c| c.is_ascii_lowercase());
    let has_upper = password.chars().any(|c| c.is_ascii_uppercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_other = password.chars().any(|c| !c.is_alphanumeric());

    for present in [has_lower, has_upper, has_digit, has_other] {
        if present {
            points += 6;
        }
    }

    if is_common(password) {
        points -= 25;
    }
    if has_sequence(password) {
        points -= 10;
    }
    if has_repeat(password) {
        points -= 8;
    }

    match points {
        i32::MIN..=12 => Score::VeryWeak,
        13..=24 => Score::Weak,
        25..=36 => Score::Fair,
        37..=48 => Score::Strong,
        _ => Score::VeryStrong,
    }
}

fn is_common(p: &str) -> bool {
    const COMMON: &[&str] = &[
        "password", "123456", "qwerty", "admin", "letmein", "welcome", "monkey", "dragon",
        "master", "abc123", "iloveyou", "111111", "000000",
    ];
    let lower = p.to_lowercase();
    COMMON.iter().any(|c| lower == *c || lower.contains(c))
}

fn has_sequence(p: &str) -> bool {
    let bytes: Vec<char> = p.chars().collect();
    bytes.windows(3).any(|w| {
        let a = w[0] as i32;
        let b = w[1] as i32;
        let c = w[2] as i32;
        (b - a == 1 && c - b == 1) || (a - b == 1 && b - c == 1)
    })
}

fn has_repeat(p: &str) -> bool {
    let chars: Vec<char> = p.chars().collect();
    chars.windows(3).any(|w| w[0] == w[1] && w[1] == w[2])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_is_weakest() {
        assert_eq!(score(""), Score::VeryWeak);
    }

    #[test]
    fn common_word_penalised() {
        assert!(score("password1") < score("Tr0ub4dor&3xyz"));
    }

    #[test]
    fn longer_beats_shorter() {
        assert!(score("aB3$dE6&gH9!kL2@") > score("aB3$dE6&"));
    }
}
