mod kinds;
mod visit;

#[derive(Default)]
pub struct Config {
    pub line_length: Option<usize>,
}

pub fn format(input: &str, config: Config) -> String {
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
    let line_ending = if source.contains("\r\n") { "\r\n" } else { "\n" };

    let mut output = source
        .lines()
        .map(|line| line.trim_end())
        .collect::<Vec<_>>()
        .join(line_ending);
    if source.ends_with('\n') {
        output.push_str(line_ending);
    }
    output
}
