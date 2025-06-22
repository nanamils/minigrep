use regex::Regex;

#[derive(Debug, PartialEq)]
pub enum MatchResult<'a> {
    Content(Vec<&'a str>),
    Line(&'a str),
}

pub trait Matcher {
    fn find<'a>(&self, line: &'a str) -> Option<MatchResult<'a>>;
}

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