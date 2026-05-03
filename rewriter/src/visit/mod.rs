mod alignment;
mod column;
mod debug;
mod quotes;
mod spacing;

pub use alignment::AlignmentVisitor;
pub use column::ColumnVisitor;
pub use debug::DebugVisitor;
pub use quotes::QuotesVisitor;
pub use spacing::SpacingVisitor;

pub trait Visitor {
    fn visit(&mut self, node: tree_sitter::Node, source: &[u8]) -> Visit;

    fn edits(self) -> Vec<Edit>;
}

pub enum Visit {
    Continue,
    Skip,
}

pub fn run_pass<V: Visitor>(
    source: &str,
    tree: &mut tree_sitter::Tree,
    parser: &mut tree_sitter::Parser,
    mut visitor: V,
) -> String {
    log::info!("Applying pass: {}", std::any::type_name::<V>());
    walk(tree.root_node(), &mut visitor, source.as_bytes());
    let edits = visitor.edits();
    let new_source = apply_edits(source, edits);
    // Parse modified tree from scratch
    let new_tree = parser.parse(&new_source, None).expect("parser language not set");
    *tree = new_tree;
    new_source
}

fn walk<T: Visitor>(root: tree_sitter::Node, visitor: &mut T, source: &[u8]) {
    let mut cursor = root.walk();
    loop {
        // Visit the current node
        let flow = visitor.visit(cursor.node(), source);

        // Skip the children if needed
        if matches!(flow, Visit::Continue) {
            if cursor.goto_first_child() {
                continue;
            }
        }

        if cursor.goto_next_sibling() {
            continue;
        }

        // Go back up the tree until we can go to the next sibling
        loop {
            if !cursor.goto_parent() {
                return;
            }
            if cursor.goto_next_sibling() {
                break;
            }
        }
    }
}

pub struct Edit {
    range: std::ops::Range<usize>,
    new_text: smol_str::SmolStr,
}

fn apply_edits(source: &str, mut edits: Vec<Edit>) -> String {
    // Apply the edits in forward order
    edits.sort_by_key(|e| e.range.start);
    for [a, b] in edits.array_windows::<2>() {
        debug_assert!(
            a.range.end <= b.range.start,
            "overlapping edits: {:?} and {:?}:\nleft: [{}] to [{}]\nright: [{}] to [{}]",
            a.range,
            b.range,
            &source[a.range.clone()],
            a.new_text,
            &source[b.range.clone()],
            b.new_text,
        );
    }

    // Pre-allocate the output string with the final size
    let delta: isize = edits
        .iter()
        .map(|e| e.new_text.len() as isize - (e.range.end - e.range.start) as isize)
        .sum();
    let capacity = (source.len() as isize + delta).max(0) as usize;
    let mut output = String::with_capacity(capacity);

    // Step forward through the source, applying edits as needed
    let mut last_index = 0;
    for edit in edits {
        output.push_str(&source[last_index..edit.range.start]);
        output.push_str(&edit.new_text);
        last_index = edit.range.end;
    }
    output.push_str(&source[last_index..]);
    output
}
