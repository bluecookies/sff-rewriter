use super::{Edit, Visit, Visitor};

#[derive(Default)]
pub struct QuotesVisitor {
    edits: Vec<Edit>,
}

const SINGLE_QUOTE: &str = "'";
const TRIPLE_QUOTE: &str = "\"\"\"";

impl Visitor for QuotesVisitor {
    fn visit(&mut self, node: tree_sitter::Node, source: &[u8]) -> Visit {
        // Only convert strings
        if node.kind() != crate::kinds::STRING {
            return Visit::Continue;
        };

        let Some(start) = node.named_child(0) else { return Visit::Continue };
        let start_text = start.utf8_text(source).unwrap_or("");

        // Skip docstrings
        if start_text.ends_with(TRIPLE_QUOTE) {
            return Visit::Skip;
        };

        // Don't convert if content contains a single quote
        let text = node.utf8_text(source).unwrap_or("");
        if text.contains(SINGLE_QUOTE) {
            return Visit::Skip;
        };

        let end = node
            .named_child(node.named_child_count() as u32 - 1)
            .expect("there is at least one child");

        // Edit just the trailing quote character
        self.edits.push(Edit {
            range: start.end_byte() - 1..start.end_byte(),
            new_text: SINGLE_QUOTE.into(),
        });

        self.edits.push(Edit {
            range: end.start_byte()..end.end_byte(),
            new_text: SINGLE_QUOTE.into(),
        });

        // Don't need to visit the children of the string node
        Visit::Skip
    }

    fn edits(self) -> Vec<Edit> {
        self.edits
    }
}
