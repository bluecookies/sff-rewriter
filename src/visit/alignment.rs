use super::{Edit, Visit, Visitor};

#[derive(Default)]
pub struct AlignmentVisitor {
    edits: Vec<Edit>,
}

impl Visitor for AlignmentVisitor {
    fn visit(&mut self, node: tree_sitter::Node, _source: &[u8]) -> Visit {
        if node.kind() != crate::kinds::ARGUMENT_LIST {
            return Visit::Continue;
        }

        // Skip empty argument lists
        let Some(child) = node.named_child(0) else { return Visit::Skip };

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

        Visit::Skip
    }

    fn edits(self) -> Vec<Edit> {
        self.edits
    }
}
