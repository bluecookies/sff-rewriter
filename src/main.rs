mod kinds;
mod visit;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&tree_sitter_python::LANGUAGE.into())?;

    let source = std::fs::read_to_string("data/test.py")?;
    let mut tree = parser.parse(&source, None).expect("parser language not set");

    let mut output = source;
    output = visit::run_pass(&output, &mut tree, &mut parser, visit::DebugVisitor::default());
    output = visit::run_pass(&output, &mut tree, &mut parser, visit::QuotesVisitor::default());
    // Format parens, brackets, braces and commas before parameter alignment, since the latter depends on the former
    output = visit::run_pass(&output, &mut tree, &mut parser, visit::SpacingVisitor::default());
    // Spacing visitor collapses multi-line lists into one line, so alignment visitor can have a canonical form based on line length
    output = visit::run_pass(&output, &mut tree, &mut parser, visit::AlignmentVisitor::default());

    // Remove trailing whitespace (including blank lines)
    output = remove_trailing_whitespace(&output);

    std::fs::write("data/test_formatted.py", output)?;

    Ok(())
}

pub fn remove_trailing_whitespace(source: &str) -> String {
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
