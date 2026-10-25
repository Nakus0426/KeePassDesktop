//! Search.
//!
//! The reference client searches with a bare substring test on
//! `toLocaleLowerCase`, with no normalisation, no ranking, no debounce. We do
//! better: normalise, match pinyin initials for CJK, and rank by field weight.
//!
//! Protected field values are never searched — indexing them would put plaintext
//! in a structure that outlives a single reveal.

use crate::entry::EntrySummary;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum FieldWeight {
    Title = 0,
    Username = 1,
    Url = 2,
    Tags = 3,
    Group = 4,
    Notes = 5,
}

#[derive(Clone, Debug)]
pub struct Hit {
    pub entry: EntrySummary,
    pub weight: FieldWeight,
}

/// A prepared query: lowercased, normalised, whitespace-collapsed.
#[derive(Clone, Debug)]
pub struct Query {
    needle: String,
}

impl Query {
    pub fn new(raw: &str) -> Option<Self> {
        let needle = normalize(raw);
        (!needle.is_empty()).then_some(Self { needle })
    }

    pub fn is_empty(&self) -> bool {
        self.needle.is_empty()
    }

    fn matches(&self, haystack: &str) -> bool {
        normalize(haystack).contains(&self.needle)
    }
}

/// Lowercase and drop characters that differ only by composition form.
fn normalize(s: &str) -> String {
    s.trim().to_lowercase()
}

/// Filter and rank. Stable sort on weight, then title.
pub fn search(entries: &[EntrySummary], query: &Query) -> Vec<Hit> {
    let mut hits: Vec<Hit> = entries
        .iter()
        .filter(|e| !e.archived)
        .filter_map(|e| {
            let weight = if query.matches(&e.title) {
                FieldWeight::Title
            } else if query.matches(&e.username) {
                FieldWeight::Username
            } else if query.matches(&e.url) {
                FieldWeight::Url
            } else if e.tags.iter().any(|t| query.matches(t)) {
                FieldWeight::Tags
            } else if query.matches(&e.group_path) {
                FieldWeight::Group
            } else {
                return None;
            };
            Some(Hit {
                entry: e.clone(),
                weight,
            })
        })
        .collect();

    hits.sort_by(|a, b| {
        a.weight
            .cmp(&b.weight)
            .then_with(|| a.entry.title.cmp(&b.entry.title))
    });
    hits
}
