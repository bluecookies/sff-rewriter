use std::path::PathBuf;

use clap::Parser;

mod diff;

const LINE_LENGTH_ENV: &str = "SFF_LINE_LENGTH";

#[derive(Parser)]
#[command(name = "sff-fmt", about = "Syntax rewriter for SFF style code")]
struct Args {
    /// File to format. Reads from stdin if not provided.
    path: Option<PathBuf>,

    /// Overwrite the file in place. Requires a path.
    #[arg(short, long, requires = "path")]
    in_place: bool,

    /// Maximum line length before breaking into multiple lines.
    #[arg(short, long)]
    line_length: Option<usize>,

    /// Only format lines that have been changed.
    /// Pass git diff output via stdin, or let the tool handle it with a path.
    #[arg(long)]
    changed_only: bool,
}

impl Args {
    fn line_length(&self) -> Option<usize> {
        self.line_length
            .or_else(|| std::env::var(LINE_LENGTH_ENV).ok().and_then(|v| v.parse().ok()))
    }
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let args = Args::parse();

    let (source, changed_lines) = match args.path {
        Some(ref path) => {
            let input = std::fs::read_to_string(path)?;

            let changed_lines = if args.changed_only {
                let diff = std::process::Command::new("git")
                    .args(["diff", "HEAD", "--unified=0", "--"])
                    .arg(path)
                    .output()?;
                let diff_str = str::from_utf8(&diff.stdout)?;
                let changed_lines = diff::parse_changed_lines(diff_str);
                Some(changed_lines)
            } else {
                None
            };
            (input, changed_lines)
        }
        None => {
            use std::io::Read;
            let mut s = String::new();
            std::io::stdin().read_to_string(&mut s)?;
            (s, None)
        }
    };

    let config = sff_formatter::Config {
        line_length: args.line_length(),
    };

    let mut output = sff_formatter::format(&source, config);

    // Filter out the set of changes to apply here instead of in the walker
    // Since the set of changed lines could change after each pass
    if let Some(lines) = changed_lines {
        let tmp = tempfile::NamedTempFile::new()?;
        std::fs::write(tmp.path(), &output)?;

        let formatter_diff = std::process::Command::new("git")
            .args(["diff", "--unified=0", "--no-index", "--"])
            .arg(args.path.as_ref().unwrap())
            .arg(tmp.path())
            .output()?;

        let formatter_diff_str = str::from_utf8(&formatter_diff.stdout)?;

        // Parse the hunks from the formatter diff, keeping only those that
        // intersect with the git changed lines
        let filtered_edits = diff::parse_hunks(formatter_diff_str)
            .into_iter()
            .filter(|(hunk, _)| lines.iter().any(|l| hunk.old_lines.contains(l)))
            .collect::<Vec<_>>();

        output = diff::apply_hunks(&source, filtered_edits);
    }

    match (&args.path, args.in_place) {
        (Some(path), true) => std::fs::write(path, output)?,
        _ => print!("{}", output),
    }

    Ok(())
}
