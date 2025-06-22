mod config;
mod matcher;
mod sink;

pub use config::Config;

use regex::RegexBuilder;
use std::{
    collections::{HashMap, HashSet}, error::Error, fs, ops::ControlFlow, path::Path
};
use {
    walkdir::WalkDir,
    matcher::{DefaultMatcher, Matcher, OnlyMatchingMatcher},
    sink::{CountSink, FilesWithMatchesSink, Sink, StandardSink}
};

struct Searcher<'a> {
    matcher: &'a dyn Matcher,
}
impl<'a> Searcher<'a> {
    fn run(&self, path: &Path, sink: &mut dyn Sink) -> Result<(), Box<dyn Error>> {
        let walker = if path.is_dir() {
            WalkDir::new(path).min_depth(1).into_iter()
        } else {
            WalkDir::new(path).max_depth(0).into_iter()
        };

        for entry in walker.filter_map(|e| e.ok()) {
            if !entry.file_type().is_file() { continue; }
            
            let file_path = entry.path();
            let contents = match fs::read_to_string(file_path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            for (i, line) in contents.lines().enumerate() {
                if let Some(match_result) = self.matcher.find(line) {
                    match sink.process_match(file_path, i + 1, match_result)? {
                        ControlFlow::Continue(_) => (),
                        ControlFlow::Break(_) => break,
                    }
                }
            }
        }
        Ok(())
    }
}


pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let re = RegexBuilder::new(&config.query)
        .case_insensitive(config.ignore_case)
        .build()?;
    
    let matcher: Box<dyn Matcher> = if config.only_matching {
        Box::new(OnlyMatchingMatcher { re: &re })
    } else {
        Box::new(DefaultMatcher { re: &re, invert_match: config.invert_match })
    };

    let mut sink: Box<dyn Sink> = if config.files_with_matches {
        Box::new(FilesWithMatchesSink { matched_files: HashSet::new() })
    } else if config.count {
        Box::new(CountSink { counts: HashMap::new() })
    } else {
        Box::new(StandardSink { config: &config, re: &re, printed_filenames: HashSet::new() })
    };

    let searcher = Searcher { matcher: &*matcher };
    searcher.run(Path::new(&config.path), &mut *sink)?;
    
    sink.finish();

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