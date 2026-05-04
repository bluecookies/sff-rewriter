use super::{Edit, Visit, Visitor};
use crate::kinds;

const DEFAULT_LINE_LENGTH: usize = 88;

pub struct AlignmentVisitor {
    line_length_threshold: usize,
    edits: Vec<Edit>,
    stack: Vec<NodeInfo>,
}

impl Default for AlignmentVisitor {
    fn default() -> Self {
        Self {
            line_length_threshold: DEFAULT_LINE_LENGTH,
            edits: Vec::new(),
            stack: Vec::new(),
        }
    }
}

impl AlignmentVisitor {
    pub fn new(config: &crate::Config) -> Self {
        Self {
            line_length_threshold: config.line_length.unwrap_or(DEFAULT_LINE_LENGTH),
            edits: Vec::new(),
            stack: Vec::new(),
        }
    }
}

struct NodeInfo {
    node_id: usize,
    list_state: Option<ListState>,
}

struct ListState {
    indent_width: usize,
    /// Original column of whichever direct named child of this list we are currently
    /// descending into. Updated as siblings are visited left-to-right, so that any
    /// nested list encountered deeper in the subtree can anchor itself relative to it.
    owning_orig_col: usize,
}

impl Visitor for AlignmentVisitor {
    fn visit(&mut self, node: tree_sitter::Node, _source: &[u8]) -> Visit {
        // Pop stack until the top is the direct parent of this node
        while !self.stack.is_empty() && node.parent().map(|p| p.id()) != self.stack.last().map(|n| n.node_id) {
            self.stack.pop();
        }

        // When entering a named node, check if it is a direct child of the nearest list
        // ancestor. If so, update that list's owning_orig_col to this node's original
        // column. This keeps track of which sibling subtree we're inside, so that nested
        // lists can compute their indentation relative to the correct anchor.
        if node.is_named() {
            let parent_id = node.parent().map(|p| p.id());
            for frame in self.stack.iter_mut().rev() {
                if let Some(ref mut ls) = frame.list_state {
                    // Only update if this node is a *direct* child of this list frame,
                    // not a deeper descendant — otherwise intermediate nodes (e.g. the
                    // argument_list inside a call) would overwrite with the wrong column.
                    if parent_id == Some(frame.node_id) {
                        ls.owning_orig_col = node.start_position().column;
                    }
                    break; // Only the nearest list ancestor is relevant
                }
            }
        }

        if !matches!(
            node.kind(),
            kinds::ARGUMENT_LIST | kinds::PARAMETERS | kinds::DICTIONARY | kinds::LIST | kinds::SET | kinds::TUPLE
        ) {
            self.stack.push(NodeInfo {
                node_id: node.id(),
                list_state: None,
            });
            return Visit::Continue;
        }

        // Skip empty lists
        let Some(first_child) = node.named_child(0) else {
            return Visit::Skip;
        };

        // Compute indentation for children of this list
        let first_child_col = first_child.start_position().column;
        let indent_width = match self.stack.iter().rev().find_map(|n| n.list_state.as_ref()) {
            Some(ls) => ls.indent_width + (first_child_col - ls.owning_orig_col),
            None => first_child_col,
        };

        // Skip if the effective line length is within threshold and this is a single line.
        // The effective length is the aligned indent plus the span from the first child to the closing delimiter,
        // i.e. where the last character would land after ancestor reformatting.
        let effective_length = indent_width + (node.end_position().column.saturating_sub(first_child_col));
        if effective_length <= self.line_length_threshold && node.start_position().row == node.end_position().row {
            return Visit::Skip;
        }

        let indent = smol_str::SmolStr::from(format!("\n{}", " ".repeat(indent_width)));

        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor).skip(1) {
            // we still need the skip(1), because the first named child still has a prev (the parentheses/brackets)
            let Some(prev) = child.prev_sibling() else { continue };
            self.edits.push(Edit {
                range: prev.end_byte()..child.start_byte(),
                new_text: indent.clone(),
            });
        }

        // Special casing for when the last element is a comment, and so the closing bracket is on a new line
        let closing = node.child(node.child_count() as u32 - 1).unwrap();
        if let Some(prev) = closing.prev_sibling()
            && prev.is_extra() {
                // closing bracket needs to be on its own line, indented to container base
                let distance = first_child
                    .start_position()
                    .column
                    .saturating_sub(node.start_position().column);
                let base_indent = indent_width.saturating_sub(distance);
                self.edits.push(Edit {
                    range: prev.end_byte()..closing.start_byte(),
                    new_text: format!("\n{}", " ".repeat(base_indent)).into(),
                });
            }

        self.stack.push(NodeInfo {
            node_id: node.id(),
            list_state: Some(ListState {
                indent_width,
                // Initialise to the first child — updated as siblings are visited
                owning_orig_col: first_child_col,
            }),
        });

        Visit::Continue
    }

    fn edits(self) -> Vec<Edit> {
        self.edits
    }
}
