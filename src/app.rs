// src/app.rs

use std::collections::HashSet;
use std::error::Error;
use std::io;
use std::path::{Path, PathBuf};

use regex::Regex;
use walkdir::WalkDir;

use crate::config::{Config, OutputMode};
use crate::search::SearcherBuilder;
pub struct App<'a> {
    config: &'a Config,
    re: &'a Regex,
    output_mode: OutputMode,
}

impl<'a> App<'a> {
    pub fn new(config: &'a Config, re: &'a Regex, output_mode: OutputMode) -> Self {
        Self { config, re, output_mode }
    }

    pub fn execute(&self) -> Result<(), Box<dyn Error>> {
        let (after_len, before_len) = self.config.get_effective_context();

        if let Some(path_str) = &self.config.path {
            let path = Path::new(path_str);
            
            let all_files = if self.output_mode == OutputMode::FilesWithoutMatch {
                Some(self.collect_all_files(path))
            } else {
                None
            };

            let mut searcher = SearcherBuilder::new(self.config, self.re)
                .build(self.output_mode, all_files);
            
            searcher.search_path(path, before_len, after_len)?;
            searcher.sink.finish();

        } else {
            if self.output_mode == OutputMode::FilesWithoutMatch {
                return Err("Error: --files-without-match is not supported for stdin.".into());
            }

            let mut searcher = SearcherBuilder::new(self.config, self.re)
                .build(self.output_mode, None);
            
            let stdin = io::stdin();
            let reader = stdin.lock();
            searcher.search_reader(reader, before_len, after_len)?;
            searcher.sink.finish();
        }

        Ok(())
    }

    fn collect_all_files(&self, path: &Path) -> HashSet<PathBuf> {
        let walker = if path.is_dir() {
            WalkDir::new(path).min_depth(1).into_iter()
        } else {
            WalkDir::new(path).max_depth(0).into_iter()
        };
        
        walker
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .map(|e| e.path().to_path_buf())
            .collect::<HashSet<PathBuf>>()
    }
}