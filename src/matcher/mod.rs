pub mod impls;

pub trait Matcher {
    fn find<'a>(&self, line: &'a str) -> Option<MatchResult<'a>>;
}

#[derive(Debug, PartialEq)]
pub enum MatchResult<'a> {
    Content(Vec<&'a str>),
    Line(&'a str),
}

