use colored::*;
use clap::Parser;
use regex::{Regex, RegexBuilder};
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

    /// Invert the sense of matching, to select non-matching lines
    #[arg(short='v', long, help = "Invert the sense of matching")]
    pub invert_match: bool,
}

#[derive(Debug, PartialEq)]
pub struct Match<'a> {
    pub line_number: usize,
    pub content: &'a str,
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let path = Path::new(&config.path);

    let highlight_regex = if !config.invert_match {
        Some(
            RegexBuilder::new(&config.query)
                .case_insensitive(config.ignore_case)
                .build()?,
        )
    } else {
        None
    };

    if path.is_dir() {
        process_directory(path, &config, highlight_regex.as_ref())?;
    } else if path.is_file() {
        process_path(path, &config, highlight_regex.as_ref(), false)?;
    } else {
        return Err(format!("'{}' is not a valid file or directory.", config.path).into());
    }

    Ok(())
}


fn process_directory(
    dir_path: &Path,
    config: &Config,
    highlight_regex: Option<&Regex>,
) -> Result<(), Box<dyn Error>> {
    for entry in WalkDir::new(dir_path).min_depth(1).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            process_path(entry.path(), config, highlight_regex, true)?;
        }
    }
    Ok(())
}

fn process_path(
    file_path: &Path,
    config: &Config,
    highlight_regex: Option<&Regex>,
    print_filename: bool,
) -> Result<(), Box<dyn Error>> {
    let contents = match fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(_) => return Ok(()),
    };

    let results = search(
        &config.query,
        &contents,
        config.ignore_case,
        config.invert_match,
    )?;

    if results.is_empty() {
        return Ok(());
    }

    if print_filename {
        println!("{}:", file_path.display().to_string().cyan());
    }

    let colored_query = config.query.red().bold().to_string();
    for line_match in results {
        let line_to_print = if let Some(re) = highlight_regex {
            re.replace_all(line_match.content, colored_query.as_str()).to_string()
        } else {
            line_match.content.to_string()
        };
        println!("{} {}", 
            format!("{: >4}:", line_match.line_number).green(), 
            line_to_print
        );
    }
    
    if print_filename {
        println!();
    }

    Ok(())
}



pub fn search<'a>(
    query: &str,
    contents: &'a str,
    ignore_case: bool,
    invert_match: bool,
) -> Result<Vec<Match<'a>>, Box<dyn Error>> {
    let regex = RegexBuilder::new(query)
        .case_insensitive(ignore_case)
        .build()?;

    let matches = contents
        .lines()
        .enumerate()
        .filter(|(_, line)| {
            if invert_match {
                !regex.is_match(line)
            } else {
                regex.is_match(line)
            }
        })
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
        assert_eq!(expected, search(query, contents, false, false).unwrap());
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
        assert_eq!(expected, search(query, contents, true, false).unwrap());
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
        assert_eq!(expected, search(query, contents, false, false).unwrap());
    }


    #[test]
    fn case_sensitive_invert() {
        let query = "duct";
        let contents = "\
Rust:
safe, fast, productive.
Pick three.
Duct tape.";
        
        let expected = vec![
            Match { line_number: 1, content: "Rust:" },
            Match { line_number: 3, content: "Pick three." },
            Match { line_number: 4, content: "Duct tape." },
        ];
        assert_eq!(expected, search(query, contents, false, true).unwrap());
    }

    #[test]
    fn case_insensitive_invert() {
        let query = "rUsT";
        let contents = "\
Rust:
safe, fast, productive.
Pick three.
Trust me.";

        let expected = vec![
            Match { line_number: 2, content: "safe, fast, productive." },
            Match { line_number: 3, content: "Pick three." },
        ];
        assert_eq!(expected, search(query, contents, true, true).unwrap());
    }
}