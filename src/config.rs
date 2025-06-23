use clap::{Args, Parser};


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputMode {
    Standard,
    Json,
    Count,
    FilesWithMatches,
    FilesWithoutMatch,
}

#[derive(Args, Debug, Clone)]
#[command(next_help_heading = "Output Modes")]
#[group(id = "output_mode_flags", multiple = false)]
pub struct OutputModeArgs {
    #[arg(long, 
        help = "Output results in JSON format", 
        group = "output_mode_flags",
        conflicts_with_all = &["files_with_matches", "count", "files_without_match"]
    )]
    pub json: bool,

    #[arg(short, long, help = "Print a count of matching lines", group = "output_mode_flags")]
    pub count: bool,

    #[arg(short = 'l', long, help = "Print only the names of files with matches", group = "output_mode_flags")]
    pub files_with_matches: bool,

    #[arg(
        long, 
        help = "Print only the names of files that DO NOT contain matches",
        group = "output_mode_flags",
        conflicts_with_all = &["json", "count", "files_with_matches"]
    )]
    pub files_without_match: bool,
}

impl TryFrom<&OutputModeArgs> for OutputMode {
    type Error = &'static str;

    fn try_from(args: &OutputModeArgs) -> Result<Self, Self::Error> {
        let dispatch_table = vec![
            (args.json, OutputMode::Json),
            (args.count, OutputMode::Count),
            (args.files_with_matches, OutputMode::FilesWithMatches),
            (args.files_without_match, OutputMode::FilesWithoutMatch),

        ];

        for (is_active, mode) in dispatch_table {
            if is_active {
                return Ok(mode);
            }
        }

        Ok(OutputMode::Standard)
    }
}

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Config {
    #[arg(help = "The pattern to search for")]
    pub query: String,
    #[arg(help = "The path to the file to search in")]
    pub path: Option<String>,
    #[command(flatten)]
    pub search: SearchOption,
    #[command(flatten)]
    pub output: OutputOption,
    #[command(flatten)]
    pub mode_args: OutputModeArgs,
}


#[derive(Args, Debug, Clone)]
#[command(next_help_heading = "Search Options")]
pub struct SearchOption {
    #[arg(short, long, help = "Case-insensitive search")]
    pub ignore_case: bool,
    #[arg(short = 'v', long, help = "Invert the sense of matching", conflicts_with = "only_matching")]
    pub invert_match: bool,
    #[arg(short, long, help = "Print only the matched parts of a line")]
    pub only_matching: bool,
}

#[derive(Args, Debug, Clone)]
#[command(next_help_heading = "Output Options")]
pub struct OutputOption {
    #[arg(short = 'A', long, value_name = "NUM", default_value_t = 0, help = "Show NUM lines of trailing context")]
    pub after_context: usize,
    #[arg(short = 'B', long, value_name = "NUM", default_value_t = 0, help = "Show NUM lines of leading context")]
    pub before_context: usize,
    #[arg(short = 'C', long, value_name = "NUM", default_value_t = 0, help = "Show NUM lines of context (A+B)")]
    pub context: usize,
    #[arg(short = 'n', long, help = "Prefix each line of output with the line number")]
    pub line_number: bool,
}

impl Config {
    pub fn get_effective_context(&self) -> (usize, usize) {
        if self.output.context > 0 {
            (self.output.context, self.output.context)
        } else {
            (self.output.after_context, self.output.before_context)
        }
    }
}