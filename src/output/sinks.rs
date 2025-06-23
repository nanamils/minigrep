use crate::{config::Config, matcher::MatchResult, output::{formatter::OutputFormatter, ContextLine, JsonContent, JsonMatch, MatchedLine, Sink}};
use colored::*;
use regex::{Captures, Regex};
use std::{
    collections::{HashMap, HashSet},
    error::Error,
    ops::ControlFlow,
    path::{PathBuf}
};

pub(crate) struct StandardSink<'a> {
    pub(crate) re: &'a Regex,
    pub(crate) formatter: OutputFormatter<'a>,
}

impl<'a> StandardSink<'a> {
    pub(crate) fn new(config: &'a Config, re: &'a Regex) -> Self {
        Self {
            re,
            formatter: OutputFormatter::new(config),
        }
    }
}

impl<'a> Sink for StandardSink<'a> {
    fn matched(&mut self, data: &MatchedLine<'_>) -> Result<ControlFlow<()>, Box<dyn Error>> {
        let prefix = self.formatter.format_prefix(data.path, data.line_number, None);

        match &data.match_result {
            MatchResult::Line(content) => {
                let highlighted_line = self.re.replace_all(content, |caps: &Captures| {
                    caps[0].red().bold().to_string()
                });
                println!("{}{}", prefix, highlighted_line);
            }
            MatchResult::Content(matches) => {
                for m in matches {
                    println!("{}{}", prefix, m.red().bold());
                }
            }
        }
        Ok(ControlFlow::Continue(()))
    }
    
    fn context(
        &mut self,
        line: &ContextLine,
    ) -> Result<ControlFlow<()>, Box<dyn Error>> {
        let prefix = self.formatter.format_prefix(&line.path, line.line_number, Some(line.kind));
        println!("{}{}", prefix, line.content);
        Ok(ControlFlow::Continue(()))
    }

    fn context_break(&mut self) -> Result<ControlFlow<()>, Box<dyn Error>> {
        println!("{}", "--".cyan());
        Ok(ControlFlow::Continue(()))
    }

    fn finish(&mut self) {}
}

#[derive(Default)]
pub(crate) struct FilesWithMatchesSink {
    pub(crate) matched_files: HashSet<PathBuf>,
}

impl Sink for FilesWithMatchesSink {
    fn matched(&mut self, data: &MatchedLine<'_>) -> Result<ControlFlow<()>, Box<dyn Error>> {
        self.matched_files.insert(data.path.to_path_buf());
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
    fn matched(&mut self, data: &MatchedLine<'_>) -> Result<ControlFlow<()>, Box<dyn Error>> {
        let path_buf = data.path.to_path_buf();
        let count = self.counts.entry(path_buf).or_insert(0);
        *count += 1;
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
    fn matched(&mut self, data: &MatchedLine<'_>) -> Result<ControlFlow<()>, Box<dyn Error>> {
       let content = match &data.match_result {
           MatchResult::Line(l) => JsonContent::Line(l.to_string()),
           MatchResult::Content(m) => JsonContent::Matches(m.iter().map(|s|s.to_string()).collect()),
       };
       self.matches.push(JsonMatch {
           path: data.path.to_path_buf(),
           line_number: data.line_number,
           content,
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

#[derive(Debug)]
pub(crate) struct FilesWithoutMatchSink {
    all_files: HashSet<PathBuf>,
    files_with_matches: HashSet<PathBuf>,
}

impl FilesWithoutMatchSink {
    pub(crate) fn new(all_files: HashSet<PathBuf>) -> Self {
        Self {
            all_files,
            files_with_matches: HashSet::new(),
        }
    }
}


impl Sink for FilesWithoutMatchSink {
    fn matched(
        &mut self,
        data: &MatchedLine<'_>,
    ) -> Result<ControlFlow<()>, Box<dyn Error>> {
        self.files_with_matches.insert(data.path.to_path_buf());
        Ok(ControlFlow::Continue(()))
    }

    fn finish(&mut self) {
        let mut files_without_matches: Vec<_> = self
            .all_files
            .difference(&self.files_with_matches)
            .collect();
        
        files_without_matches.sort();

        for path in files_without_matches {
            println!("{}", path.display().to_string().cyan());
        }
    }
}