//! Integration tests for the dashboard generator.
//!
//! These are written before the implementation (TDD). They are also the
//! living specification: every behavior the dashboard claims to provide is
//! pinned down by at least one assertion below. If you add a new signal,
//! add a test first.

use std::fs;

use xtask::dashboard::{
    DashboardData, ModuleStats, SmellCounts, TestCorpusEntry, count_smells,
    find_orphan_modules, render, splice,
};

// ---------------------------------------------------------------------------
// Orphan-module detection
//
// An "orphan" is a .rs file under src/ that no other source file references
// via a `mod NAME;` declaration. lib.rs, main.rs, and mod.rs are entry
// points and are never reported as orphans.
//
// The detector is a regex-based heuristic, not a full Rust parser. The tests
// pin down the cases we currently support; corner cases that fail (e.g.
// `#[path = "..."]` attributes) are documented and expected to be a known
// limitation.
// ---------------------------------------------------------------------------

#[test]
fn detects_simple_orphan_alongside_a_referenced_sibling() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path();
    fs::write(src.join("main.rs"), "mod used;\nfn main() {}\n").unwrap();
    fs::write(src.join("used.rs"), "// used module\n").unwrap();
    fs::write(src.join("orphan.rs"), "// orphan module\n").unwrap();

    let orphans = find_orphan_modules(src).unwrap();
    let names: Vec<_> = orphans
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    assert_eq!(names, vec!["orphan.rs"]);
}

#[test]
fn entry_points_are_never_flagged_as_orphans() {
    // A repository that only contains main.rs (no other modules at all)
    // must produce an empty orphan list — main.rs is the entry point.
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path();
    fs::write(src.join("main.rs"), "fn main() {}\n").unwrap();
    fs::write(src.join("lib.rs"), "// library root\n").unwrap();

    assert!(find_orphan_modules(src).unwrap().is_empty());
}

#[test]
fn all_visibility_forms_count_as_references() {
    // `pub mod`, `pub(crate) mod`, `pub(super) mod`, and bare `mod` all
    // bring a file into the module tree. Missing any of these would cause
    // false-positive orphans on real codebases.
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path();
    fs::write(
        src.join("lib.rs"),
        "pub mod a;\npub(crate) mod b;\npub(super) mod c;\nmod d;\n",
    )
    .unwrap();
    for name in ["a.rs", "b.rs", "c.rs", "d.rs"] {
        fs::write(src.join(name), "").unwrap();
    }

    assert!(find_orphan_modules(src).unwrap().is_empty());
}

#[test]
fn inline_mod_blocks_do_not_count_as_file_references() {
    // `mod foo { ... }` (note the brace, no semicolon) declares an inline
    // module that lives in the current file. The file `src/foo.rs` is NOT
    // brought in by this form and is therefore still an orphan.
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path();
    fs::write(src.join("main.rs"), "mod foo { fn inner() {} }\n").unwrap();
    fs::write(src.join("foo.rs"), "// orphan, despite the inline mod above\n").unwrap();

    let orphans = find_orphan_modules(src).unwrap();
    let names: Vec<_> = orphans
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    assert_eq!(names, vec!["foo.rs"]);
}

#[test]
fn references_from_any_source_file_count_not_just_entry_points() {
    // A mod declaration anywhere in the tree should disarm an orphan, not
    // only those at lib.rs / main.rs.
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path();
    fs::write(src.join("lib.rs"), "pub mod parent;\n").unwrap();
    fs::write(src.join("parent.rs"), "pub mod child;\n").unwrap();
    fs::write(src.join("child.rs"), "// referenced transitively\n").unwrap();

    assert!(find_orphan_modules(src).unwrap().is_empty());
}

// ---------------------------------------------------------------------------
// Smell counting (heuristic grep)
//
// We deliberately do not parse Rust; we count substring occurrences. This
// is admitted as a heuristic in the rendered output. The tests pin down
// the exact substrings counted.
// ---------------------------------------------------------------------------

#[test]
fn count_smells_counts_each_pattern() {
    let source = "let x = result.unwrap();\nlet y = vec.clone();\npanic!(\"oops\");\nlet z = other_vec.clone();\nunsafe { *p = 1; }\n";
    let counts = count_smells(source);
    assert_eq!(counts.unwrap_count, 1);
    assert_eq!(counts.clone_count, 2);
    assert_eq!(counts.panic_count, 1);
    assert_eq!(counts.unsafe_count, 1);
}

#[test]
fn count_smells_is_empty_for_clean_source() {
    let counts = count_smells("fn main() { println!(\"hello\"); }\n");
    assert_eq!(counts.unwrap_count, 0);
    assert_eq!(counts.clone_count, 0);
    assert_eq!(counts.panic_count, 0);
    assert_eq!(counts.unsafe_count, 0);
}

// ---------------------------------------------------------------------------
// README splice operation
//
// The dashboard occupies the region between BEGIN/END markers in README.md.
// The splice operation must be idempotent: running it twice with the same
// new content yields the same output.
// ---------------------------------------------------------------------------

const MARK_BEGIN: &str = "<!-- DASHBOARD:BEGIN -->";
const MARK_END: &str = "<!-- DASHBOARD:END -->";

#[test]
fn splice_replaces_existing_content_between_markers() {
    let readme = format!(
        "# Title\n\n{MARK_BEGIN}\nOLD CONTENT\n{MARK_END}\n\nFooter\n"
    );
    let out = splice(&readme, "NEW CONTENT").unwrap();
    let expected = format!(
        "# Title\n\n{MARK_BEGIN}\nNEW CONTENT\n{MARK_END}\n\nFooter\n"
    );
    assert_eq!(out, expected);
}

#[test]
fn splice_appends_a_section_when_markers_are_absent() {
    let readme = "# Title\n\nSome content.\n";
    let out = splice(readme, "DASHBOARD CONTENT").unwrap();
    assert!(out.starts_with("# Title"));
    assert!(out.contains(MARK_BEGIN));
    assert!(out.contains("DASHBOARD CONTENT"));
    assert!(out.contains(MARK_END));
    // The original content must still be present.
    assert!(out.contains("Some content."));
}

#[test]
fn splice_is_idempotent_under_repeated_invocation() {
    // Running splice twice with the same payload must produce the same file.
    // This is the property that lets a pre-commit hook be safe.
    let readme = "# Title\n\nBody.\n";
    let once = splice(readme, "PAYLOAD").unwrap();
    let twice = splice(&once, "PAYLOAD").unwrap();
    assert_eq!(once, twice);
}

#[test]
fn splice_errors_on_orphan_begin_marker() {
    // A BEGIN with no matching END is a corrupt README and we refuse to
    // guess the boundary. Better to fail loudly than to silently swallow
    // the rest of the document.
    let readme = format!("# Title\n\n{MARK_BEGIN}\nstuff\n\nno end marker\n");
    assert!(splice(&readme, "NEW").is_err());
}

// ---------------------------------------------------------------------------
// Rendering
//
// We assert on the presence of specific signal values, not on byte-for-byte
// markdown layout. This lets us tune formatting without breaking tests, as
// long as the truth content is preserved.
// ---------------------------------------------------------------------------

#[test]
fn render_emits_every_signal_provided_in_the_input() {
    let data = DashboardData {
        modules: vec![
            ModuleStats { name: "parser.rs".into(), loc: 3668 },
            ModuleStats { name: "lexer.rs".into(), loc: 606 },
        ],
        smells: SmellCounts {
            unwrap_count: 34,
            clone_count: 279,
            panic_count: 22,
            unsafe_count: 0,
        },
        orphans: vec!["gen.rs".into()],
        test_corpus: vec![
            TestCorpusEntry { glob: "test/valid/*.c".into(), count: 89 },
            TestCorpusEntry { glob: "test/invalid/*.c".into(), count: 12 },
        ],
        rustc_version: "rustc 1.94.1 (e408947bf 2026-03-25)".into(),
        platform: "Linux 6.18.5 x86_64".into(),
        commit: "abc1234".into(),
        dirty: false,
    };

    let out = render(&data);

    // Module table
    assert!(out.contains("parser.rs"), "module name missing");
    assert!(out.contains("3668"), "LOC missing");
    assert!(out.contains("lexer.rs"));

    // Smells
    assert!(out.contains("34"), "unwrap count missing");
    assert!(out.contains("279"), "clone count missing");

    // Orphans called out by name
    assert!(out.contains("gen.rs"), "orphan name missing");

    // Test corpus glob and count
    assert!(out.contains("test/valid/*.c"));
    assert!(out.contains("89"));

    // Provenance: must always render commit + rustc version
    assert!(out.contains("abc1234"));
    assert!(out.contains("rustc 1.94.1"));
    assert!(out.contains("Linux"));
}

#[test]
fn render_flags_dirty_working_tree() {
    // If the working tree has uncommitted changes, the dashboard must say
    // so. A dashboard claiming to reflect commit X when X is not what's on
    // disk is a lie — exactly the failure mode we are building this tool
    // to prevent.
    let data = DashboardData {
        modules: vec![],
        smells: SmellCounts::default(),
        orphans: vec![],
        test_corpus: vec![],
        rustc_version: "rustc 1.94.1".into(),
        platform: "Linux x86_64".into(),
        commit: "abc1234".into(),
        dirty: true,
    };
    let out = render(&data);
    assert!(
        out.to_lowercase().contains("dirty") || out.contains("uncommitted"),
        "dirty flag must surface visibly in the rendered dashboard, got:\n{out}"
    );
}

#[test]
fn render_renders_no_orphans_section_when_empty() {
    // The orphans section is a warning. Don't print "Orphans: (none)" —
    // print nothing, so the dashboard stays quiet when things are healthy.
    let data = DashboardData {
        modules: vec![ModuleStats { name: "lib.rs".into(), loc: 10 }],
        smells: SmellCounts::default(),
        orphans: vec![],
        test_corpus: vec![],
        rustc_version: "rustc 1.94.1".into(),
        platform: "Linux x86_64".into(),
        commit: "abc1234".into(),
        dirty: false,
    };
    let out = render(&data);
    assert!(!out.to_lowercase().contains("orphan"), "should not mention orphans when there are none, got:\n{out}");
}
