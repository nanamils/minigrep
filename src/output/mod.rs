pub mod formatter;
pub mod sinks;

use std::{error::Error, ops::ControlFlow, path::{Path, PathBuf}};
use serde::Serialize;
use crate::matcher::MatchResult;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextKind {
    Before,
    After,
}

#[derive(Debug, Clone)]
pub struct ContextLine {
    pub path: PathBuf,
    pub line_number: usize,
    pub content: String,
    pub kind: ContextKind,
}

#[derive(Debug)]
pub struct MatchedLine<'a> {
    pub path: &'a Path,
    pub line_number: usize,
    pub match_result: MatchResult<'a>,
}

#[derive(Serialize, Debug)]
#[serde(untagged)]
pub(crate) enum JsonContent {
    Line(String),
    Matches(Vec<String>),
}

#[derive(Serialize, Debug)]
pub(crate) struct JsonMatch {
    path: PathBuf,
    line_number: usize,
    content: JsonContent,
}

pub trait Sink {
    fn matched(
        &mut self,
        data: &MatchedLine<'_>,
    ) -> Result<ControlFlow<()>, Box<dyn Error>>;

    fn context(
        &mut self,
        _line: &ContextLine,
    ) -> Result<ControlFlow<()>, Box<dyn Error>> {
        Ok(ControlFlow::Continue(()))
    }

    fn context_break(&mut self) -> Result<ControlFlow<()>, Box<dyn Error>> {
        Ok(ControlFlow::Continue(()))
    }

    fn finish(&mut self);
}
