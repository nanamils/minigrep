use regex::Regex;

use crate::matcher::{MatchResult, Matcher};

pub(crate) struct DefaultMatcher<'a> {
    pub(crate) re: &'a Regex,
    pub(crate) invert_match: bool,
}

impl<'a> Matcher for DefaultMatcher<'a> {
    fn find<'b>(&self, line: &'b str) -> Option<MatchResult<'b>> {
        let is_match = self.re.is_match(line);
        if (is_match && !self.invert_match) || (!is_match && self.invert_match) {
            Some(MatchResult::Line(line))
        } else {
            None
        }
    }
}

pub(crate) struct OnlyMatchingMatcher<'a> {
    pub(crate) re: &'a Regex,
}

impl<'a> Matcher for OnlyMatchingMatcher<'a> {
    fn find<'b>(&self, line: &'b str) -> Option<MatchResult<'b>> {
        let matches: Vec<&str> = self.re.find_iter(line).map(|m| m.as_str()).collect();
        if matches.is_empty() {
            None
        } else {
            Some(MatchResult::Content(matches))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;

    #[test]
    fn test_default_matcher() {
        let re = Regex::new("test").unwrap();
        let matcher = DefaultMatcher { re: &re, invert_match: false };
        let line = "this is a test line";
        if let Some(MatchResult::Line(content)) = matcher.find(line) {
            assert_eq!(content, "this is a test line");
        } else {
            panic!("Expected a match");
        }
        let line_no_match = "no match here";
        assert!(matcher.find(line_no_match).is_none());
    }

    #[test]
    fn test_default_matcher_invert() {
        let re = Regex::new("test").unwrap();
        let matcher = DefaultMatcher { re: &re, invert_match: true };

        let line_no_match = "no match here";
        if let Some(MatchResult::Line(content)) = matcher.find(line_no_match) {
            assert_eq!(content, "no match here");
        } else {
            panic!("Expected a match");
        }

        let line = "this is a test line";
        assert!(matcher.find(line).is_none());
    }

    #[test]
    fn test_only_matching_matcher() {
        let re = Regex::new(r"\d+").unwrap();
        let matcher = OnlyMatchingMatcher { re: &re };
        
        let line = "hello 123 world 456";
        if let Some(MatchResult::Content(matches)) = matcher.find(line) {
            assert_eq!(matches, vec!["123", "456"]);
        } else {
            panic!("Expected matches");
        }

        let line_no_match = "no numbers here";
        assert!(matcher.find(line_no_match).is_none());
    }
}