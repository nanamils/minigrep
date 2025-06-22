use std::path::Path;
use colored::*;
use crate::Config;

pub struct OutputFormatter<'a> {
    config: &'a Config,
}

impl<'a> OutputFormatter<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self { config }
    }

    pub fn format_prefix(&self, file_path: &Path, line_number: usize) -> String {
        let mut prefix = String::new();
        
        let is_multi_file = self.config.path.as_ref().map_or(false, |p| Path::new(p).is_dir());
        if is_multi_file {
            prefix.push_str(&format!("{}:", file_path.display().to_string().cyan()));
        }

        // if self.config.line_number {
            prefix.push_str(&format!("{}:", line_number.to_string().green()));
        // }

        prefix
    }
}