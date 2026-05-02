mod visit;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&tree_sitter_python::LANGUAGE.into())?;

    let source = std::fs::read_to_string("data/test.py")?;
    let tree = parser.parse(&source, None).expect("parser language not set");
    let root_node = tree.root_node();

    let mut output = source;
    output = visit::run_pass(&output, root_node, visit::DebugVisitor);
    output = visit::run_pass(&output, root_node, visit::QuotesVisitor::default());
    // Format parens, brackets and braces before parameter alignment, since the latter depends on the former
    output = visit::run_pass(&output, root_node, visit::BracketsVisitor::default());

    std::fs::write("data/test_formatted.py", output)?;

    Ok(())
}
