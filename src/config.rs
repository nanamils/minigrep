use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Config {
    #[arg(help = "The pattern to search for")]
    pub query: String,
    #[arg(help = "The path to the file to search in")]
    pub path: Option<String>,
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
    #[arg(short = 'n', long, help = "Prefix each line of output with the line number")]
    pub line_number: bool,
    #[arg(long, 
        help = "Output results in JSON format",
        conflicts_with_all = &["files_with_matches", "count"]
    )]
    pub json: bool,
    #[arg(short = 'A', long, value_name = "NUM", default_value_t = 0, help = "Show NUM lines of trailing context")]
    pub after_context: usize,
    #[arg(short = 'B', long, value_name = "NUM", default_value_t = 0, help = "Show NUM lines of leading context")]
    pub before_context: usize,
    #[arg(short = 'C', long, value_name = "NUM", default_value_t = 0, help = "Show NUM lines of context (A+B)")]
    pub context: usize,
}

impl Config {
    pub fn get_effective_context(&self) -> (usize, usize) {
        if self.context > 0 {
            (self.context, self.context)
        } else {
            (self.after_context, self.before_context)
        }
    }
}