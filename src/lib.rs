use clap::Parser;
use regex::RegexBuilder;
use std::{fs, error::Error};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Config {
    /// The pattern to search for (can be a regex)
    #[arg(help = "The pattern to search for")]
    pub query: String,

    /// The path to the file to search in
    #[arg(help = "The path to the file to search")]
    pub file_path: String,

    /// Perform a case-insensitive search
    #[arg(short, long, help = "Case-insensitive search")]
    pub ignore_case: bool,
}



pub fn search<'a>(query: &str, contents: &'a str, ignore_case: bool) -> Vec<&'a str> {
    let regex = RegexBuilder::new(query)
        .case_insensitive(ignore_case)
        .build()
        .expect("Invalid regex pattern");

    contents
        .lines()
        .filter(|line| regex.is_match(line))
        .collect()
}




pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let contents = fs::read_to_string(config.file_path)?;

    let results = search(&config.query, &contents, config.ignore_case);

    for line in results {
        println!("{line}");
    }

    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_result() {
        let query = "duct";
        let contents = "\
Rust:
safe, fast, productive.
Pick three.
Duct tape.";
        assert_eq!(vec!["safe, fast, productive."], search(query, contents, false));
    }

    #[test]
    fn case_insensitive_search() {
        let query = "rUsT";
        let contents = "\
Rust:
safe, fast, productive.
Pick three.
Trust me.";
        assert_eq!(vec!["Rust:", "Trust me."], search(query, contents, true));
    }

    #[test]
    fn regex_search_pattern() {
        let query = r"^\w{4}:$";
        let contents = "\
Rust:
safe, fast, productive.
Pick three.
Four:";

        assert_eq!(vec!["Rust:", "Four:"], search(query, contents, false));
    }
}