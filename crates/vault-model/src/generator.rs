//! Password generation. Three modes, parameter-compatible with the reference
//! client so a user's habits carry over.

use secrecy::SecretString;

#[derive(Clone, Debug)]
pub struct RandomParams {
    pub length: usize,
    pub lowercase: bool,
    pub uppercase: bool,
    pub digits: bool,
    pub symbols: bool,
    /// Exclude look-alike characters (Il1O0, oO, etc).
    pub exclude_ambiguous: bool,
    pub min_digits: usize,
    pub min_symbols: usize,
}

impl Default for RandomParams {
    fn default() -> Self {
        Self {
            length: 16,
            lowercase: true,
            uppercase: true,
            digits: true,
            symbols: true,
            exclude_ambiguous: true,
            min_digits: 1,
            min_symbols: 1,
        }
    }
}

#[derive(Clone, Debug)]
pub struct MemorableParams {
    pub words: usize,
    pub separator: char,
    pub capitalize: bool,
    pub include_digit: bool,
}

impl Default for MemorableParams {
    fn default() -> Self {
        Self {
            words: 4,
            separator: '-',
            capitalize: true,
            include_digit: true,
        }
    }
}

#[derive(Clone, Debug)]
pub struct PinParams {
    pub length: usize,
    /// Reject runs (1234) and repeats (1111).
    pub avoid_patterns: bool,
}

impl Default for PinParams {
    fn default() -> Self {
        Self {
            length: 6,
            avoid_patterns: true,
        }
    }
}

/// Parameters clamped to the ranges the UI offers, matching the reference
/// client: random 8-20, memorable 3-8 words, PIN 4-12.
pub fn clamp_random(p: &mut RandomParams) {
    p.length = p.length.clamp(8, 64);
}

pub fn clamp_memorable(p: &mut MemorableParams) {
    p.words = p.words.clamp(3, 8);
}

pub fn clamp_pin(p: &mut PinParams) {
    p.length = p.length.clamp(4, 12);
}

/// Generate. Implementations live in `random`/`memorable`/`pin` once the RNG
/// choice is settled; this signature is the contract.
pub trait Generator {
    fn generate(&self) -> SecretString;
}
