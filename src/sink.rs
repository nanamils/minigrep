use crate::{config::Config, matcher::MatchResult, formatter::OutputFormatter};
use colored::*;
use regex::Regex;
use std::{
    collections::{HashMap, HashSet},
    error::Error,
    ops::ControlFlow,
    path::{Path, PathBuf}
};
use serde::Serialize;

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
    fn process_match(&mut self, file_path: &Path, line_number: usize, result: MatchResult) -> Result<ControlFlow<()>, Box<dyn std::error::Error>>;
    fn finish(&mut self);
}

pub(crate) struct StandardSink<'a> {
    pub(crate) config: &'a Config,
    pub(crate) re: &'a Regex,
    pub(crate) formatter: OutputFormatter<'a>,
}
impl<'a> StandardSink<'a> {
    pub(crate) fn new(config: &'a Config, re: &'a Regex) -> Self {
        Self {
            config,
            re,
            formatter: OutputFormatter::new(config),
        }
    }
}

impl<'a> Sink for StandardSink<'a> {
    fn process_match(
        &mut self,
        file_path: &Path,
        line_number: usize,
        result: MatchResult,
    ) -> Result<ControlFlow<()>, Box<dyn Error>> {
        let prefix = self.formatter.format_prefix(file_path, line_number);

        match result {
            MatchResult::Line(content) => {
                let line_to_print = if !self.config.invert_match {
                    self.re.replace_all(content, self.config.query.red().bold().to_string().as_str())
                        .to_string()
                } else {
                    content.to_string()
                };
                println!("{}{}", prefix, line_to_print);
            }
            MatchResult::Content(matches) => {
                for m in matches {
                    println!("{}{}", prefix, m.red().bold());
                }
            }
        }
        Ok(ControlFlow::Continue(()))
    }

    fn finish(&mut self) { /* 无操作 */ }
}

#[derive(Default)]
pub(crate) struct FilesWithMatchesSink {
    pub(crate) matched_files: HashSet<PathBuf>,
}

impl Sink for FilesWithMatchesSink {
    fn process_match(&mut self, file_path: &Path, _ln: usize, _res: MatchResult) -> Result<ControlFlow<()>, Box<dyn Error>> {
        self.matched_files.insert(file_path.to_path_buf());
        Ok(ControlFlow::Break(()))
    }
    fn finish(&mut self) {
        let mut sorted_files: Vec<_> = self.matched_files.iter().collect();
        sorted_files.sort();
        for file in sorted_files {
            println!("{}", file.display().to_string().cyan());
        }
    }
}
#[derive(Default)]
pub(crate) struct CountSink {
    pub(crate) counts: HashMap<PathBuf, u64>,
}

impl Sink for CountSink {
    fn process_match(&mut self, file_path: &Path, _line_number: usize, result: MatchResult) -> Result<ControlFlow<()>, Box<dyn std::error::Error>> {
        let should_count = match result {
            MatchResult::Line(_) => true,
            MatchResult::Content(matches) => !matches.is_empty(),
        };

        if should_count {
            let count = self.counts.entry(file_path.to_path_buf()).or_insert(0);
            *count += 1;
        }
        
        Ok(ControlFlow::Continue(()))
    }

    fn finish(&mut self) {
        let mut total = 0;
        let mut sorted_paths: Vec<_> = self.counts.keys().collect();
        sorted_paths.sort();

        let print_total = self.counts.len() > 1;

        for path in sorted_paths {
            if let Some(count) = self.counts.get(path) {
                println!("{}:{}", path.display().to_string().cyan(), count);
                total += count;
            }
        }

        if print_total {
            println!("Total: {}", total);
        }
    }
}
#[derive(Default)]
pub(crate) struct JsonSink {
    matches: Vec<JsonMatch>,
}

impl Sink for JsonSink {
    fn process_match(
        &mut self,
        file_path: &Path,
        line_number: usize,
        result: MatchResult,
    ) -> Result<ControlFlow<()>, Box<dyn Error>> {
        let json_content = match result {
            MatchResult::Line(line) => JsonContent::Line(line.to_owned()),
            MatchResult::Content(matches) => {
                let owned_matches: Vec<String> = matches.iter().map(|s| s.to_string()).collect();
                JsonContent::Matches(owned_matches)
            }
        };

        self.matches.push(JsonMatch {
            path: file_path.to_path_buf(),
            line_number,
            content: json_content,
        });

        Ok(ControlFlow::Continue(()))
    }

    fn finish(&mut self) {
        if self.matches.is_empty() {
            println!("[]");
            return;
        }

        match serde_json::to_string_pretty(&self.matches) {
            Ok(json_string) => println!("{}", json_string),
            Err(e) => eprintln!("Error serializing to JSON: {}", e),
        }
    }
}