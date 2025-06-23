use std::{collections::VecDeque, error::Error, path::{Path, PathBuf}};

use crate::output::{ContextKind, ContextLine, MatchedLine, Sink};


pub struct ContextManager<'a, 's> {
    sink: &'s mut (dyn Sink + 'a),
    before_len: usize,
    after_len: usize,
    before_buffer: VecDeque<(usize, String)>,
    after_countdown: usize,
    last_match_line_num: usize,
    path: PathBuf,
}

impl<'a, 's> ContextManager<'a, 's> {
    pub fn new(
        sink: &'s mut (dyn Sink + 'a), 
        before_len: usize, 
        after_len: usize,
        path: &Path
    ) -> Self {
        Self {
            sink,
            before_len,
            after_len,
            before_buffer: VecDeque::with_capacity(before_len),
            after_countdown: 0,
            last_match_line_num: 0,
            path: path.to_path_buf(),
        }
    }

    pub fn handle_match(
        &mut self, 
        line_num: usize, 
        match_result: crate::matcher::MatchResult
    ) -> Result<(), Box<dyn Error>> {
        let context_enabled = self.before_len > 0 || self.after_len > 0;

        if context_enabled && self.last_match_line_num > 0 && line_num > self.last_match_line_num + self.after_len + 1 {
            let _ = self.sink.context_break()?;
        }

        for (b_line_num, b_content) in &self.before_buffer {
            if *b_line_num > self.last_match_line_num {
                let _ = self.sink.context(&ContextLine {
                   path: self.path.clone(),
                   line_number: *b_line_num,
                   content: b_content.clone(),
                   kind: ContextKind::Before,
               })?;
            }
        }
        self.before_buffer.clear();

        let _ = self.sink.matched(&MatchedLine {
            path: &self.path,
            line_number: line_num,
            match_result,
        })?;

        self.last_match_line_num = line_num;
        self.after_countdown = self.after_len;

        Ok(())
    }

    pub fn handle_non_match(
        &mut self,
        line_num: usize,
        line_content: String
    ) -> Result<(), Box<dyn Error>> {
        if self.after_countdown > 0 {
            let _ = self.sink.context(&ContextLine {
                path: self.path.clone(),
                line_number: line_num,
                content: line_content.clone(),
                kind: ContextKind::After,
            })?;
            self.last_match_line_num = line_num;
            self.after_countdown -= 1;
        }
        
        if self.before_len > 0 {
            if self.before_buffer.len() == self.before_len {
                self.before_buffer.pop_front();
            }
            self.before_buffer.push_back((line_num, line_content));
        }
        Ok(())
    }
}