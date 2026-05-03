use std::path::PathBuf;

use clap::Parser;

mod kinds;
mod visit;

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

struct Config {
    line_length: Option<usize>,
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let args = Args::parse();

    let source = match args.path {
        Some(ref path) => std::fs::read_to_string(path)?,
        None => {
            use std::io::Read;
            let mut s = String::new();
            std::io::stdin().read_to_string(&mut s)?;
            s
        }
    };

    if args.changed_only {
        todo!("not handled yet")
    }

    let config = Config {
        line_length: args.line_length(),
    };

    let output = format(&source, config);

    match (&args.path, args.in_place) {
        (Some(path), true) => std::fs::write(path, output)?,
        _ => print!("{}", output),
    }

    Ok(())
}

fn format(input: &str, config: Config) -> String {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_python::LANGUAGE.into())
        .expect("incompatible language version");

    let mut tree = parser.parse(input, None).expect("parser language not set");

    let mut output;

    output = visit::run_pass(input, &mut tree, &mut parser, visit::DebugVisitor::default());
    output = visit::run_pass(&output, &mut tree, &mut parser, visit::QuotesVisitor::default());
    // Format parens, brackets, braces and commas before parameter alignment, since the latter depends on the former
    output = visit::run_pass(&output, &mut tree, &mut parser, visit::SpacingVisitor::default());
    // Spacing visitor collapses multi-line lists into one line, so alignment visitor can have a canonical form based on line length
    output = visit::run_pass(&output, &mut tree, &mut parser, visit::AlignmentVisitor::new(&config));

    // Remove trailing whitespace (including blank lines)
    output = remove_trailing_whitespace(&output);

    output
}

fn remove_trailing_whitespace(source: &str) -> String {
    let mut output = source
        .lines()
        .map(|line| line.trim_end())
        .collect::<Vec<_>>()
        .join("\n");
    if source.ends_with('\n') {
        output.push('\n');
    }
    output
}
