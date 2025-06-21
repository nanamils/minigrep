use colored::*;
use clap::Parser;
use regex::RegexBuilder;
use std::{error::Error, fs, path::Path};
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Config {
    /// The pattern to search for (can be a regex)
    #[arg(help = "The pattern to search for")]
    pub query: String,

    /// The path to the file to search in
    #[arg(help = "The path to the file to search")]
    pub path: String,

    /// Perform a case-insensitive search
    #[arg(short, long, help = "Case-insensitive search")]
    pub ignore_case: bool,
}

#[derive(Debug, PartialEq)]
pub struct Match<'a> {
    pub line_number: usize,
    pub content: &'a str,
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let path = Path::new(&config.path);
    let highlight_regex = RegexBuilder::new(&config.query)
        .case_insensitive(config.ignore_case)
        .build()?;
    let colored_query = config.query.red().bold().to_string();

    if path.is_dir() {
        for entry in WalkDir::new(path).min_depth(1).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                let file_path = entry.path();
                if let Ok(contents) = fs::read_to_string(file_path) {
                    let results = search(&config.query, &contents, config.ignore_case)?;
                    if !results.is_empty() {
                        println!("{}:", file_path.display().to_string().cyan());
                        for line_match in results {
                            let highlighted_line = highlight_regex.replace_all(
                                line_match.content, 
                                colored_query.as_str()
                            );

                            println!("{} {}", 
                                format!("{: >4}:", line_match.line_number).green(), 
                                highlighted_line
                            );
                        }
                        println!();
                    }
                }
            }
        }
    } else if path.is_file() {
        let contents = fs::read_to_string(path)?;
        let results = search(&config.query, &contents, config.ignore_case)?;
        for line_match in results {
            let highlighted_line = highlight_regex.replace_all(
                line_match.content, 
                colored_query.as_str()
            );
            println!("{} {}", 
                format!("{: >4}:", line_match.line_number).green(), 
                highlighted_line
            );
        }
    } else {
        return Err(format!("'{}' is not a valid file or directory.", config.path).into());
    }

    Ok(())
}


pub fn search<'a>(
    query: &str,
    contents: &'a str,
    ignore_case: bool,
) -> Result<Vec<Match<'a>>, Box<dyn Error>> {
    let regex = RegexBuilder::new(query)
        .case_insensitive(ignore_case)
        .build()?;
    let matches = contents
        .lines()
        .enumerate()
        .filter(|(_, line)| regex.is_match(line))
        .map(|(i, line)| Match {
            line_number: i + 1,
            content: line,
        })
        .collect();

    Ok(matches)
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
        
        let expected = vec![
            Match { line_number: 2, content: "safe, fast, productive." }
        ];
        assert_eq!(expected, search(query, contents, false).unwrap());
    }

    #[test]
    fn case_insensitive_search() {
        let query = "rUsT";
        let contents = "\
Rust:
safe, fast, productive.
Pick three.
Trust me.";

        let expected = vec![
            Match { line_number: 1, content: "Rust:" },
            Match { line_number: 4, content: "Trust me." },
        ];
        assert_eq!(expected, search(query, contents, true).unwrap());
    }

    #[test]
    fn regex_search_pattern() {
        let query = r"^\w{4}:$";
        let contents = "\
Rust:
safe, fast, productive.
Pick three.
Four:";

        let expected = vec![
            Match { line_number: 1, content: "Rust:" },
            Match { line_number: 4, content: "Four:" },
        ];
        assert_eq!(expected, search(query, contents, false).unwrap());
    }
}