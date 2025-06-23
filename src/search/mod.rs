pub mod context;

use std::{collections::HashSet, error::Error, fs::File, io::{BufRead, BufReader}, path::{Path, PathBuf}};

use regex::Regex;
use walkdir::WalkDir;

use crate::{
    config::OutputMode, fs::{is_binary, is_hidden}, matcher::{
        impls::{
            DefaultMatcher,
            OnlyMatchingMatcher
        }, Matcher}, output::{
        sinks::{CountSink, FilesWithMatchesSink, FilesWithoutMatchSink, JsonSink, StandardSink
        }, Sink}, search::context::ContextManager, Config
    };

pub(crate) struct SearcherBuilder<'a> {
    config: &'a Config,
    re: &'a Regex,
}

impl<'a> SearcherBuilder<'a> {
    pub(crate) fn new(config: &'a Config, re: &'a Regex) -> Self {
        Self { config, re }
    }

    fn build_matcher(&self) -> Box<dyn Matcher + 'a> {
        if self.config.search.only_matching {
            Box::new(OnlyMatchingMatcher { re: self.re })
        } else {
            Box::new(DefaultMatcher {
                re: self.re,
                invert_match: self.config.search.invert_match,
            })
        }
    }

    fn build_sink(
        &self,
        mode: OutputMode,
        all_files: Option<HashSet<PathBuf>>,
    ) -> Box<dyn Sink + 'a> {
        match mode {
            OutputMode::Standard => Box::new(StandardSink::new(self.config, self.re)),
            OutputMode::Json => Box::new(JsonSink::default()),
            OutputMode::Count => Box::new(CountSink::default()),
            OutputMode::FilesWithMatches => Box::new(FilesWithMatchesSink::default()),
            OutputMode::FilesWithoutMatch => {
                let files = all_files.expect("List of all files is required for --files-without-match");
                Box::new(FilesWithoutMatchSink::new(files))
            }
        }
    }

    pub(crate) fn build(
        self,
        mode: OutputMode,
        all_files: Option<HashSet<PathBuf>>,
    ) -> Searcher<'a> {
        Searcher {
            matcher: self.build_matcher(),
            sink: self.build_sink(mode, all_files),
        }
    }
}

pub(crate) struct Searcher<'a> {
    pub(crate) matcher: Box<dyn Matcher + 'a>,
    pub(crate) sink: Box<dyn Sink + 'a>,
}

impl<'a> Searcher<'a> {
    fn search_stream<R: BufRead>(
        &mut self,
        reader: R,
        path: &Path,
        before_len: usize,
        after_len: usize,
    ) -> Result<(), Box<dyn Error>> {
        let mut context_manager = ContextManager::new(
            self.sink.as_mut(),
            before_len,
            after_len,
            path
        );

        for (i, line_res) in reader.lines().enumerate() {
            let line_num = i + 1;
            let line_content = line_res?;

            if let Some(match_result) = self.matcher.find(&line_content) {
                context_manager.handle_match(line_num, match_result)?;
            } else {
                context_manager.handle_non_match(line_num, line_content)?;
            }
        }
        Ok(())
    }

    pub(crate) fn search_reader<R: BufRead>(
        &mut self,
        reader: R,
        before_len: usize,
        after_len: usize,
    ) -> Result<(), Box<dyn Error>> {
        self.search_stream(reader, Path::new("stdin"), before_len, after_len)
    }

    pub(crate) fn search_path(
        &mut self,
        path: &Path,
        before_len: usize,
        after_len: usize,
    ) -> Result<(), Box<dyn Error>> {
        let mut builder = WalkDir::new(path);
        if path.is_dir() {
            builder = builder.min_depth(1);
        } else {
            builder = builder.max_depth(0);
        }
    
        let walker = builder.into_iter()
            .filter_entry(|e| !is_hidden(e));
    
        for entry_result in walker {
            let entry = match entry_result {
                Ok(e) => e,
                Err(e) => {
                    eprintln!("Failed to access path: {}", e);
                    continue;
                }
            };
    
            if !entry.file_type().is_file() {
                continue;
            }
    
            let file_path = entry.path();
    
            if is_binary(file_path).unwrap_or(true) {
                continue;
            }
            
            let file = match File::open(file_path) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("Failed to open {}: {}", file_path.display(), e);
                    continue;
                }
            };
            let reader = BufReader::new(file);
    
            self.search_stream(reader, file_path, before_len, after_len)?;
        }
        Ok(())
    }

}
