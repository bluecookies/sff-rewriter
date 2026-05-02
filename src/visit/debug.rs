use super::{Edit, Visit, Visitor};

pub struct DebugVisitor;

impl Visitor for DebugVisitor {
    fn visit(&mut self, node: tree_sitter::Node, source: &[u8]) -> Visit {
        let kind_fmt = if node.is_named() {
            if node.child_count() == 0 {
                format!("\x1b[1;32m<{}>\x1b[0m", node.kind()) // bold green for named leaves
            } else {
                format!("\x1b[1;36m<{}>\x1b[0m", node.kind()) // bold cyan for named inner nodes
            }
        } else {
            if node.child_count() == 0 {
                format!("\x1b[2;33m<{}>\x1b[0m", node.kind()) // dim yellow for unnamed leaves (punctuation)
            } else {
                format!("\x1b[2;37m<{}>\x1b[0m", node.kind()) // dim white for unnamed inner nodes
            }
        };
        let source_spanned = node
            .utf8_text(source)
            .unwrap_or("\x1b[3m<invalid utf-8 or out of range>\x1b[0m");
        log::debug!("{}: {}", kind_fmt, source_spanned,);
        Visit::Continue
    }

    fn edits(self) -> Vec<Edit> {
        Vec::new()
    }
}
