use std::collections::BTreeSet;
use std::ops::Range;

pub struct Hunk {
    pub old_lines: Range<usize>,
    pub new_lines: Range<usize>,
}

pub fn parse_changed_lines(diff: &str) -> BTreeSet<usize> {
    log::debug!("Parsing changed lines from git diff");
    let mut changed_lines = BTreeSet::new();

    for line in diff.lines() {
        if let Some(hunk) = parse_hunk(line) {
            changed_lines.extend(hunk.new_lines);
        }
    }

    for line in changed_lines.iter() {
        log::debug!("Changed line: {}", line);
    }
    log::debug!("{} changed lines.", changed_lines.len());

    changed_lines
}

pub fn parse_hunks(diff: &str) -> Vec<(Hunk, Vec<String>)> {
    let mut hunks = Vec::new();
    let mut current: Option<(Hunk, Vec<String>)> = None;

    for line in diff.lines() {
        if let Some(hunk) = parse_hunk(line) {
            if let Some(h) = current.take() {
                hunks.push(h);
            }
            current = Some((hunk, Vec::new()));
        } else if let Some((_, ref mut lines)) = current
            && let Some(new_line) = line.strip_prefix('+') {
                lines.push(new_line.to_string());
            }
    }

    if let Some(h) = current.take() {
        hunks.push(h);
    }

    hunks
}

pub fn apply_hunks(source: &str, hunks: Vec<(Hunk, Vec<String>)>) -> String {
    log::debug!("Applying {} hunks", hunks.len());
    let source_lines: Vec<&str> = source.lines().collect();
    let mut output = Vec::new();
    let mut source_line = 0;

    for (hunk, new_lines) in hunks.iter() {
        output.extend_from_slice(&source_lines[source_line..hunk.old_lines.start]);
        source_line = hunk.old_lines.end;
        output.extend(new_lines.iter().map(String::as_str));
    }

    output.extend_from_slice(&source_lines[source_line..]);
    let mut result = output.join("\n");
    if source.ends_with('\n') {
        result.push('\n');
    }
    result
}

fn parse_hunk(line: &str) -> Option<Hunk> {
    let rest = line.strip_prefix("@@ -")?;
    let (old_part, rest) = rest.split_once(" +")?;
    let (new_part, _) = rest.split_once(" @@")?;

    let parse_range = |s: &str| -> Option<Range<usize>> {
        let (start, count): (usize, usize) = match s.split_once(',') {
            Some((s, c)) => (s.parse().ok()?, c.parse().ok()?),
            None => (s.parse().ok()?, 1usize),
        };
        Some((start - 1)..(start - 1 + count))
    };

    Some(Hunk {
        old_lines: parse_range(old_part)?,
        new_lines: parse_range(new_part)?,
    })
}
