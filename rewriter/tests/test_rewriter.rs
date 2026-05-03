use std::path::Path;

use similar::{ChangeTag, TextDiff};

fn run_test(name: &str) {
    let input = std::fs::read_to_string(Path::new("tests/input").join(name).with_extension("py"))
        .expect("input file not found");

    let expected = std::fs::read_to_string(Path::new("tests/output").join(name).with_extension("py"))
        .expect("expected file not found");

    let output = sff_formatter::format(&input, sff_formatter::Config::default());

    if output != expected {
        let diff = TextDiff::from_lines(&expected, &output);
        for hunk in diff.unified_diff().context_radius(3).iter_hunks() {
            eprintln!("\x1b[36m{}\x1b[0m", hunk.header());
            for change in hunk.iter_changes() {
                let (sign, color) = match change.tag() {
                    ChangeTag::Delete => ("-", "\x1b[31m"),
                    ChangeTag::Insert => ("+", "\x1b[32m"),
                    ChangeTag::Equal => (" ", "\x1b[0m"),
                };
                let with_line_end = change.value().replace('\r', "␍").replace('\n', "␊\n");
                eprint!("{}{}{}\x1b[0m", color, sign, with_line_end);
            }
        }
        panic!("output did not match expected for '{}'", name);
    }
}

#[test]
fn test_quotes() {
    run_test("quotes");
}

#[test]
fn test_spacing() {
    run_test("parens");
}

#[test]
fn test_alignment() {
    run_test("alignment");
}

#[test]
fn test_comments_alignment() {
    run_test("comments");
}

#[test]
fn test_columns() {
    run_test("columns");
}
