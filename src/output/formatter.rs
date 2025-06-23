use std::path::Path;
use colored::*;
use crate::{output::ContextKind, Config};

pub struct OutputFormatter<'a> {
    config: &'a Config,
}

impl<'a> OutputFormatter<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self { config }
    }

    pub(crate) fn format_prefix(
        &self, 
        file_path: &Path, 
        line_number: usize,
        context_kind: Option<ContextKind> 
    ) -> String {
        let mut prefix = String::new();
        let is_multi_file_context = self.config.path.as_ref().map_or(false, |p| Path::new(p).is_dir());

        if is_multi_file_context {
            prefix.push_str(&format!("{}:", file_path.display().to_string().cyan()));
        }
        
        let separator = match context_kind {
            Some(_) => "-",
            None => ":",
        };
        prefix.push_str(&format!("{}{}", line_number.to_string().green(), separator));

        prefix
    }
}