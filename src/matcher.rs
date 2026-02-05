use crate::model::Entry;
use nucleo_matcher::{Matcher, Utf32Str};

pub struct FuzzyMatcher {
    matcher: Matcher,
}

impl Default for FuzzyMatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl FuzzyMatcher {
    pub fn new() -> Self {
        Self {
            matcher: Matcher::new(nucleo_matcher::Config::DEFAULT),
        }
    }

    pub fn match_entries(&mut self, query: &str, entries: &mut [Entry]) {
        let pattern = nucleo_matcher::pattern::Pattern::parse(query, nucleo_matcher::pattern::CaseMatching::Smart, nucleo_matcher::pattern::Normalization::Smart);
        
        let mut buf = Vec::new(); // Reusable buffer if needed, though for simple scoring we might just loop

        // Nucleo is designed for large lists, we can use score_pattern for one-off or implement the full pattern matching
        // For simplicity here, we iterate and score.
        
        for entry in entries.iter_mut() {
            let haystack = Utf32Str::new(&entry.name, &mut buf);
            if let Some(score) = pattern.score(haystack, &mut self.matcher) {
                entry.score = score as i64;
            } else {
                entry.score = -1;
            }
        }
        
        // Filter out non-matches and sort
        // Note: The caller should filter entries with score < 0
    }
}
