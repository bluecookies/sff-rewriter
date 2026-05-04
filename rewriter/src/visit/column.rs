use super::{Edit, Visit, Visitor};
use crate::kinds;

pub struct ColumnVisitor {
    column_spacing: usize,
    edits: Vec<Edit>,
}

impl Default for ColumnVisitor {
    fn default() -> Self {
        ColumnVisitor {
            column_spacing: 10,
            edits: Vec::new(),
        }
    }
}

#[derive(Copy, Clone)]
enum ColumnAlignment {
    Left,
    #[expect(unused)]
    Right,
}

struct Column {
    width: usize,
    alignment: ColumnAlignment,
}

struct Row<'a> {
    elements: Vec<tree_sitter::Node<'a>>,
    trailing_comment: Option<tree_sitter::Node<'a>>,
}

impl ColumnVisitor {
    fn try_collect_rows<'a>(&self, node: tree_sitter::Node<'a>) -> Option<Vec<Row<'a>>> {
        // 1. Check all named children (ignoring comments) are either all tuples or all lists
        // 2. Check all rows have the same element count
        // 3. Pair each row with its trailing comment (next sibling if is_extra())
        // 4. Abort if any row has multiple trailing comments
        // Returns None if any check fails
        let mut rows = Vec::new();
        let mut cursor = node.walk();
        let children: Vec<_> = node.named_children(&mut cursor).collect();

        // Filter out comments to check row kinds
        let row_nodes: Vec<_> = children.iter().filter(|n| !n.is_extra()).collect();

        if row_nodes.is_empty() {
            return None;
        };

        // All rows must be the same kind (tuple or list)
        let row_kind = row_nodes[0].kind();
        if !matches!(row_kind, kinds::TUPLE | kinds::LIST) {
            return None;
        };
        if !row_nodes.iter().all(|n| n.kind() == row_kind) {
            return None;
        };

        // All rows must have the same number of named children
        let num_cols = row_nodes[0].named_child_count();
        if !row_nodes.iter().all(|n| n.named_child_count() == num_cols) {
            return None;
        };
        if num_cols == 0 {
            return None;
        };

        // Collect row node ids first so we can use them as sentinels
        let row_ids: std::collections::BTreeSet<usize> = row_nodes.iter().map(|n| n.id()).collect();

        // Pair each row with its trailing comment
        for row_node in row_nodes {
            let mut trailing_comment = None;
            let mut comment_count = 0;

            let mut sib = row_node.next_sibling();
            while let Some(s) = sib {
                if row_ids.contains(&s.id()) {
                    // Hit the next row, stop
                    break;
                }
                if s.is_extra() {
                    comment_count += 1;
                    if comment_count > 1 {
                        return None;
                    };
                    trailing_comment = Some(s);
                } else {
                    debug_assert!(
                        !s.is_named(),
                        "unexpected named non-extra node between rows: {:?}",
                        s.kind()
                    );
                }
                sib = s.next_sibling();
            }

            let mut elem_cursor = row_node.walk();
            let elements = row_node.named_children(&mut elem_cursor).collect();

            rows.push(Row {
                elements,
                trailing_comment,
            });
        }

        Some(rows)
    }

    fn compute_columns(&self, rows: &[Row], source: &[u8]) -> Vec<Column> {
        // For each column index, find max element width across all rows
        // and determine alignment
        let num_cols = rows[0].elements.len();
        (0..num_cols)
            .map(|col| {
                let width = rows
                    .iter()
                    .map(|row| row.elements[col].utf8_text(source).unwrap_or("").len())
                    .max()
                    .unwrap_or(0)
                    .max(1);
                Column {
                    width,
                    alignment: ColumnAlignment::Left, // hardcoded for now
                }
            })
            .collect()
    }

    fn format_row(&mut self, row: &Row, columns: &[Column], source: &[u8], is_last: bool) {
        for (i, (element, col)) in row.elements.iter().zip(columns.iter()).enumerate() {
            let text = element.utf8_text(source).unwrap_or("");
            let end = if i == columns.len() - 1 { " " } else { "" };
            let padded: smol_str::SmolStr = match col.alignment {
                ColumnAlignment::Left => format!(" {:<width$}{end}", text, width = col.width).into(),
                ColumnAlignment::Right => format!(" {:>width$}{end}", text, width = col.width).into(),
            };
            let prev = element.prev_sibling().expect("no prev sibling");
            let next = element.next_sibling().expect("no next sibling");
            self.edits.push(Edit {
                range: prev.end_byte()..next.start_byte(),
                new_text: padded,
            });
        }

        // Skip last row - we handle that separately
        if let Some(comment) = row.trailing_comment
            && !is_last
        {
            let gap_start = comment
                .prev_sibling()
                .expect("comment should always have previous sibling")
                .end_byte();

            let padding = " ".repeat(self.column_spacing).into();
            self.edits.push(Edit {
                range: gap_start..comment.start_byte(),
                new_text: padding,
            });
        }
    }
}

impl Visitor for ColumnVisitor {
    fn visit(&mut self, node: tree_sitter::Node, source: &[u8]) -> Visit {
        if node.kind() != kinds::LIST {
            return Visit::Continue;
        };

        let Some(rows) = self.try_collect_rows(node) else { return Visit::Continue };
        let columns = self.compute_columns(&rows, source);

        for (i, row) in rows.iter().enumerate() {
            self.format_row(&row, &columns, source, rows.len() - 1 == i);
        }

        // Handle comment on last row correctly
        let last_row = rows.last().expect("empty list");
        if let Some(comment) = last_row.trailing_comment {
            let closing = node
                .child(node.child_count() as u32 - 1)
                .expect("list should have a closing bracket");

            // Remove the entire gap between the real last element and the end of the list
            let last_element = comment
                .prev_sibling()
                .expect("comment should always have a previous sibling");

            let gap_start = last_element.end_byte();

            self.edits.push(Edit {
                range: gap_start..closing.start_byte(),
                new_text: " ".into(),
            });

            // Recreate it after the closing ] of the list node
            let comment_text = comment.utf8_text(source).unwrap_or("");

            // Take into account last row has ] and maybe no comma - hardcode
            let extra = if last_element.kind() == "," { 2 } else { 1 };
            let padding = self.column_spacing.saturating_sub(extra);

            self.edits.push(Edit {
                range: node.end_byte()..node.end_byte(),
                new_text: format!("{}{}", " ".repeat(padding), comment_text).into(),
            });
        }

        Visit::Continue
    }

    fn edits(self) -> Vec<Edit> {
        self.edits
    }
}
