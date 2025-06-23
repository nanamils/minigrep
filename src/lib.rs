mod config;
mod matcher;
mod output;
mod app;
mod search;
mod fs;
pub use config::Config;
use crate::{app::App, config::OutputMode}; 
use regex::RegexBuilder;
use std::error::Error;

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let re = RegexBuilder::new(&config.query)
        .case_insensitive(config.search.ignore_case)
        .build()?;
    
    let output_mode = OutputMode::try_from(&config.mode_args)?;

    let app = App::new(&config, &re, output_mode);
    
    app.execute()
}

