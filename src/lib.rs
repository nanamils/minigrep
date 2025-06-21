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
    #[arg(short='v', long, help = "Invert the sense of matching", conflicts_with = "only_matching")]
    pub invert_match: bool,

    /// Only show filenames of files that contain matches
    #[arg(short='l', long, help = "Print only the names of files with matches")]
    pub files_with_matches: bool,

    /// Print only the matched (non-empty) parts of a matching line
    #[arg(short, long, help = "Print only the matched parts of a line")]
    pub only_matching: bool,
}

#[derive(Debug, PartialEq)]
pub struct Match<'a> {
    pub line_number: usize,
    pub content: &'a str,
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let path = Path::new(&config.path);

    let main_regex = if !config.files_with_matches {
        Some(
            RegexBuilder::new(&config.query)
                .case_insensitive(config.ignore_case)
                .build()?,
        )
    } else {
        None
    };

    if path.is_dir() {
        process_directory(path, &config, main_regex.as_ref())?;
    } else if path.is_file() {
        process_path(path, &config, main_regex.as_ref(), false)?;
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
    main_regex: Option<&Regex>,
    _is_dir_context: bool,
) -> Result<(), Box<dyn Error>> {
    let contents = match fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(_) => return Ok(()),
    };

    // -l 标志的优先级最高
    if config.files_with_matches {
        let results = search(&config.query, &contents, config.ignore_case, config.invert_match)?;
        if !results.is_empty() {
            println!("{}", file_path.display().to_string().cyan());
        }
        return Ok(());
    }
    
    // 如果不是 -l 模式，我们可以直接使用 main_regex 来查找匹配
    // 这避免了在 search 和 process_path 中重复编译 regex
    let re = main_regex.unwrap(); // 在非-l模式下，main_regex一定是Some
    
    let print_filename_prefix = _is_dir_context;
    let mut printed_filename = false;

    for (i, line) in contents.lines().enumerate() {
        let line_number = i + 1;

        if config.only_matching {
            // --- -o 模式的逻辑 ---
            for mat in re.find_iter(line) {
                if !printed_filename && print_filename_prefix {
                    println!("{}:", file_path.display().to_string().cyan());
                    printed_filename = true;
                }
                println!("{} {}",
                    format!("{: >4}:", line_number).green(),
                    mat.as_str().red().bold()
                );
            }
        } else {
            // --- 正常和 -v 模式的逻辑 ---
            let is_match = re.is_match(line);
            if (is_match && !config.invert_match) || (!is_match && config.invert_match) {
                 if !printed_filename && print_filename_prefix {
                    println!("{}:", file_path.display().to_string().cyan());
                    printed_filename = true;
                }
                let line_to_print = if !config.invert_match {
                    re.replace_all(line, config.query.red().bold().to_string().as_str()).to_string()
                } else {
                    line.to_string()
                };
                println!("{} {}", 
                    format!("{: >4}:", line_number).green(), 
                    line_to_print
                );
            }
        }
    }

    if printed_filename {
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