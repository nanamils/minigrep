mod config;
mod matcher;
mod sink;
pub mod formatter;

pub use config::Config;

use regex::{Regex, RegexBuilder};
use std::{
    error::Error, fs, ops::ControlFlow, path::Path, io::{self, BufRead}
};

use {
    walkdir::WalkDir,
    matcher::{DefaultMatcher, Matcher, OnlyMatchingMatcher},
    sink::{JsonSink, CountSink, FilesWithMatchesSink, Sink, StandardSink}
};
struct SearcherBuilder<'a> {
    config: &'a Config,
    re: &'a Regex,
}

impl<'a> SearcherBuilder<'a> {
    fn new(config: &'a Config, re: &'a Regex) -> Self {
        Self { config, re }
    }

    fn build_matcher(&self) -> Box<dyn Matcher + 'a> {
        if self.config.only_matching {
            Box::new(OnlyMatchingMatcher { re: self.re })
        } else {
            Box::new(DefaultMatcher {
                re: self.re,
                invert_match: self.config.invert_match,
            })
        }
    }

    fn build_sink(&self) -> Box<dyn Sink + 'a> {
        let sink_factories: Vec<(
            Box<dyn Fn(&Config) -> bool>,
            Box<dyn Fn(&'a Config, &'a Regex) -> Box<dyn Sink + 'a>>,
        )> = vec![
            (
                Box::new(|c| c.json),
                Box::new(|_, _| Box::new(JsonSink::default())),
            ),
            (
                Box::new(|c| c.files_with_matches),
                Box::new(|_, _| Box::new(FilesWithMatchesSink::default())),
            ),
            (
                Box::new(|c| c.count), 
                Box::new(|_, _| Box::new(CountSink::default()))
            ),
            // *** 未来添加新的 Sink 只需要在这里加一行！ ***
        ];

        sink_factories
            .iter()
            .find(|(predicate, _)| predicate(self.config))
            .map(|(_, constructor)| constructor(self.config, self.re))
            .unwrap_or_else(|| {
                Box::new(StandardSink::new(self.config, self.re))
            })
    }

    fn build(self) -> Searcher<'a> {
        Searcher {
            matcher: self.build_matcher(),
            sink: self.build_sink(),
        }
    }
}

struct Searcher<'a> {
    matcher: Box<dyn Matcher + 'a>,
    sink: Box<dyn Sink + 'a>,
}

impl<'a> Searcher<'a> {
    fn search_path(&mut self, path: &Path) -> Result<(), Box<dyn Error>> {
        let walker = if path.is_dir() {
            WalkDir::new(path).min_depth(1).into_iter()
        } else {
            WalkDir::new(path).max_depth(0).into_iter()
        };

        for entry in walker.filter_map(|e| e.ok()) {
            if !entry.file_type().is_file() {
                continue;
            }

            let file_path = entry.path();
            let contents = match fs::read_to_string(file_path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            for (i, line) in contents.lines().enumerate() {
                if let Some(match_result) = self.matcher.find(line) {
                    match self.sink.process_match(file_path, i + 1, match_result)? {
                        ControlFlow::Continue(_) => (),
                        ControlFlow::Break(_) => break,
                    }
                }
            }
        }
        Ok(())
    }

    fn search_reader<R: BufRead>(&mut self, reader: R) -> Result<(), Box<dyn Error>> {
        for (i, line_result) in reader.lines().enumerate() {
            let line = line_result?;
            if let Some(match_result) = self.matcher.find(&line) {
                let _ = self.sink.process_match(Path::new("stdin"), i + 1, match_result)?;
            }
        }
        Ok(())
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let re = RegexBuilder::new(&config.query)
        .case_insensitive(config.ignore_case)
        .build()?;

    let mut searcher = SearcherBuilder::new(&config, &re).build();

    if let Some(path_str) = &config.path {
        searcher.search_path(Path::new(path_str))?;
    } else {
        let stdin = io::stdin();
        let reader = stdin.lock();
        searcher.search_reader(reader)?;
    }

    searcher.sink.finish();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::matcher::{DefaultMatcher, MatchResult, OnlyMatchingMatcher};
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