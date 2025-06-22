use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Config {
    #[arg(help = "The pattern to search for")]
    pub query: String,
    #[arg(help = "The path to the file to search in")]
    pub path: String,
    #[arg(short, long, help = "Case-insensitive search")]
    pub ignore_case: bool,
    #[arg(short = 'v', long, help = "Invert the sense of matching", conflicts_with = "only_matching")]
    pub invert_match: bool,
    #[arg(short = 'l', long, help = "Print only the names of files with matches", conflicts_with = "count")]
    pub files_with_matches: bool,
    #[arg(short, long, help = "Print only the matched parts of a line")]
    pub only_matching: bool,
    #[arg(short, long, help = "Print a count of matching lines")]
    pub count: bool,
}