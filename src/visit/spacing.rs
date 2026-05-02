use super::{Edit, Visit, Visitor};

#[derive(Default)]
pub struct SpacingVisitor {
    edits: Vec<Edit>,
}

impl Visitor for SpacingVisitor {
    fn visit(&mut self, node: tree_sitter::Node, _source: &[u8]) -> Visit {
        if let Some(parent) = node.parent() {
            // Skip the opening and closing braces of an f-string interpolation,
            // since the spacing there is significant (sometimes, AFAIK its when you use =)
            if parent.kind() == "interpolation" {
                let is_first = parent.child(0).map(|n| n.id() == node.id()).unwrap_or(false);
                let is_last = parent
                    .child(parent.child_count() as u32 - 1)
                    .map(|n| n.id() == node.id())
                    .unwrap_or(false);
                if is_first || is_last {
                    return Visit::Continue;
                };
            }
        }

        match node.kind() {
            "(" | "[" | "{" => {
                let Some(next) = node.next_sibling() else { return Visit::Continue };
                // Skip (), [], {} - works because brackets cannot mismatch
                if matches!(next.kind(), ")" | "]" | "}") {
                    return Visit::Continue;
                };
                self.edits.push(Edit {
                    range: node.end_byte()..next.start_byte(),
                    new_text: " ".into(),
                });
            }
            ")" | "]" | "}" => {
                let Some(prev) = node.prev_sibling() else { return Visit::Continue };
                // Skip (), [], {} - works because brackets cannot mismatch
                // Also skip commas before closing brackets, since that is handled already
                if matches!(prev.kind(), "(" | "[" | "{" | ",") {
                    return Visit::Continue;
                };
                self.edits.push(Edit {
                    range: prev.end_byte()..node.start_byte(),
                    new_text: " ".into(),
                });
            }
            "," => {
                let Some(next) = node.next_sibling() else { return Visit::Continue };
                self.edits.push(Edit {
                    range: node.end_byte()..next.start_byte(),
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
