use super::{Edit, Visit, Visitor};
use crate::kinds;

pub struct AlignmentVisitor {
    line_length_threshold: usize,
    edits: Vec<Edit>,
}

impl Default for AlignmentVisitor {
    fn default() -> Self {
        Self {
            line_length_threshold: 88, // PEP 8 recommends 79, but 88 is the default in Black and allows for some flexibility
            edits: Vec::new(),
        }
    }
}

impl Visitor for AlignmentVisitor {
    fn visit(&mut self, node: tree_sitter::Node, _source: &[u8]) -> Visit {
        if !matches!(
            node.kind(),
            kinds::ARGUMENT_LIST | kinds::PARAMETERS | kinds::DICTIONARY | kinds::LIST | kinds::SET | kinds::TUPLE
        ) {
            return Visit::Continue;
        }

        // Skip empty argument lists
        let Some(child) = node.named_child(0) else { return Visit::Skip };

        // Skip if line length is less than threshold
        if node.end_position().column <= self.line_length_threshold {
            return Visit::Skip;
        }

        // Get the indentation of the first element and align the rest to that
        let indent_width = child.start_position().column;
        let indent = smol_str::SmolStr::from(format!("\n{}", " ".repeat(indent_width)));

        let mut cursor = node.walk();
        // Skip the first child since that is the reference for alignment
        for child in node.named_children(&mut cursor).skip(1) {
            self.edits.push(Edit {
                range: child.start_byte()..child.start_byte(),
                new_text: indent.clone(),
            });
        }

        // Continue in case we need to format a nested dict
        Visit::Continue
    }

    fn edits(self) -> Vec<Edit> {
        self.edits
    }
}
