use super::{Edit, Visit, Visitor};

#[derive(Default)]
pub struct ParensVisitor {
    edits: Vec<Edit>,
}

impl Visitor for ParensVisitor {
    fn visit(&mut self, node: tree_sitter::Node, _source: &[u8]) -> Visit {
        match node.kind() {
            "(" => {
                let Some(next) = node.next_sibling() else { return Visit::Continue };
                // Skip parentheses like ()
                if next.kind() == ")" {
                    return Visit::Continue;
                };
                self.edits.push(Edit {
                    range: node.end_byte()..next.start_byte(),
                    new_text: " ".into(),
                });
            }
            ")" => {
                let Some(prev) = node.prev_sibling() else { return Visit::Continue };
                // Skip parentheses like ()
                if prev.kind() == "(" {
                    return Visit::Continue;
                };
                self.edits.push(Edit {
                    range: prev.end_byte()..node.start_byte(),
                    new_text: " ".into(),
                });
            }
            _ => {}
        }
        Visit::Continue
    }

    fn edits(self) -> Vec<Edit> {
        self.edits
    }
}
