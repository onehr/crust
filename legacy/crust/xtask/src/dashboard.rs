//! Dashboard generation.
//!
//! The public entry points are:
//!
//! * [`find_orphan_modules`] — scan a `src/` directory, return files that
//!   no `mod NAME;` declaration references. Surfaces silent dead code.
//! * [`count_smells`]        — count occurrences of well-known Rust
//!   anti-patterns in a source string (heuristic, substring-based).
//! * [`render`]              — turn a [`DashboardData`] into the markdown
//!   that will live between the dashboard markers in `README.md`.
//! * [`splice`]              — insert (or replace) the dashboard region
//!   inside an existing `README.md`, idempotently.
//!
//! All of these functions are pure: they take inputs, return values, do no
//! I/O. The CLI in `main.rs` is responsible for reading files, running
//! `git`/`rustc`/`uname`, and writing the result back to disk. Keeping the
//! library pure is what makes the tests under `xtask/tests/dashboard.rs`
//! deterministic and fast.

use std::collections::HashSet;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use regex::Regex;

// ===========================================================================
// Public data model
// ===========================================================================

/// All inputs needed to render the dashboard. Constructed by the CLI from
/// filesystem and subprocess output; consumed by [`render`].
#[derive(Debug, Clone)]
pub struct DashboardData {
    pub modules: Vec<ModuleStats>,
    pub smells: SmellCounts,
    pub orphans: Vec<String>,
    pub test_corpus: Vec<TestCorpusEntry>,
    pub rustc_version: String,
    pub platform: String,
    pub commit: String,
    /// True if the working tree has uncommitted changes at the time of
    /// generation. Surfaced visibly in the rendered output, because a
    /// dashboard that quietly attributes its numbers to a commit that
    /// doesn't match what's on disk is a lie.
    pub dirty: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleStats {
    pub name: String,
    pub loc: usize,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SmellCounts {
    pub unwrap_count: usize,
    pub clone_count: usize,
    pub panic_count: usize,
    pub unsafe_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestCorpusEntry {
    pub glob: String,
    pub count: usize,
}

// ===========================================================================
// Orphan-module detection
// ===========================================================================

/// Module-declaration regex.
///
/// Matches lines of the form `mod NAME;`, optionally prefixed with a
/// visibility modifier. The trailing semicolon is required: `mod NAME { ... }`
/// (inline module) does NOT pull in a file and must therefore not match.
///
/// Limitations (documented, not bugs to fix in v1):
///
/// * False positives from comments or string literals (e.g. `// mod foo;`
///   will be counted as a reference). We accept these because they err on
///   the side of "looks referenced" rather than "wrongly flagged as
///   orphan" — a quiet dashboard is better than a noisy one that cries
///   wolf.
/// * `#[path = "other.rs"] mod foo;` will count `foo` as referenced even
///   though the file on disk is `other.rs`. This pattern is rare in
///   practice.
static MOD_DECL: LazyLock<Regex> = LazyLock::new(|| {
    // ^                 anchored at line start (re-applied per line)
    // \s*               leading whitespace
    // (?:pub(?:\([^)]*\))?\s+)?    optional `pub` / `pub(crate)` / `pub(super)` / `pub(in ...)`
    // mod\s+            the `mod` keyword
    // ([a-zA-Z_]\w*)    captured identifier
    // \s*;              terminating semicolon
    Regex::new(r"^\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+([a-zA-Z_]\w*)\s*;").unwrap()
});

/// Files that are always part of the crate root, regardless of whether any
/// other file references them by `mod`.
///
/// `mod.rs` is included for the (rare in this codebase, but common in the
/// wild) sub-directory module pattern. We do not currently recurse into
/// subdirectories, but listing it here keeps the detector honest if/when
/// we do.
const ENTRY_POINTS: &[&str] = &["main.rs", "lib.rs", "mod.rs"];

/// Return the `.rs` files under `src_dir` that no `mod` declaration brings
/// into the module tree. Entry points (`main.rs`, `lib.rs`, `mod.rs`) are
/// never reported.
///
/// The scan is non-recursive: only the immediate children of `src_dir` are
/// examined. This matches crust's flat module layout. A future revision
/// that handles `src/foo/mod.rs` sub-trees would need to walk the tree and
/// resolve `mod` declarations relative to each containing file.
pub fn find_orphan_modules(src_dir: &Path) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let mut rs_files: Vec<PathBuf> = Vec::new();
    let mut referenced: HashSet<String> = HashSet::new();

    for entry in fs::read_dir(src_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        rs_files.push(path.clone());

        // Scan this file for `mod NAME;` declarations and accumulate the
        // names. We do this line-by-line because the regex is line-anchored.
        let text = fs::read_to_string(&path)?;
        for line in text.lines() {
            if let Some(caps) = MOD_DECL.captures(line) {
                referenced.insert(caps[1].to_string());
            }
        }
    }

    let mut orphans = Vec::new();
    for path in rs_files {
        let file_name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_string();
        if ENTRY_POINTS.contains(&file_name.as_str()) {
            continue;
        }
        // `foo.rs` is referenced iff `mod foo;` appears in some file.
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        if !referenced.contains(stem) {
            orphans.push(path);
        }
    }
    orphans.sort();
    Ok(orphans)
}

// ===========================================================================
// Smell counting (substring grep)
// ===========================================================================

/// Count occurrences of well-known anti-patterns in a Rust source string.
///
/// This is admitted as a *heuristic*: we are not parsing Rust. A `.clone()`
/// inside a string literal will be counted, as will a `// .unwrap()` in a
/// comment. The honest move is to label these counts as "heuristic grep"
/// in the rendered output and let the reader judge.
///
/// The patterns counted are deliberately the ones that most often signal
/// learner-Rust smells (and so are the most actionable when triaging an
/// old toy project):
///
/// | Pattern   | Why it's interesting                                       |
/// |-----------|------------------------------------------------------------|
/// | `.unwrap()` | Hides error paths; usually replaceable with `?`.         |
/// | `.clone()`  | Often masks a lifetime/borrow issue the author punted on.|
/// | `panic!()`  | Abnormal exit; usually should be `Result::Err` upstream. |
/// | `unsafe`    | Memory-safety boundary; auditing point.                  |
pub fn count_smells(source: &str) -> SmellCounts {
    // `count_overlapping` would be overkill — these substrings are
    // distinct enough that the standard non-overlapping count matches
    // intent. (Two consecutive `.clone()` calls are two matches.)
    SmellCounts {
        unwrap_count: source.matches(".unwrap()").count(),
        clone_count: source.matches(".clone()").count(),
        panic_count: source.matches("panic!").count(),
        unsafe_count: source.matches("unsafe").count(),
    }
}

// ===========================================================================
// README splice
// ===========================================================================

/// Marker that opens the auto-generated region of `README.md`.
///
/// We use HTML comments so the markers themselves are invisible when the
/// README is rendered on GitHub.
pub const MARKER_BEGIN: &str = "<!-- DASHBOARD:BEGIN -->";

/// Marker that closes the auto-generated region of `README.md`.
pub const MARKER_END: &str = "<!-- DASHBOARD:END -->";

/// Insert `new_content` into `readme` between [`MARKER_BEGIN`] and
/// [`MARKER_END`], replacing whatever was there.
///
/// If the markers are absent, a fresh dashboard section is appended to the
/// end of `readme` (preceded by a blank line for readability).
///
/// If only one of the markers is present, this is a sign of a corrupted or
/// hand-edited dashboard region. We refuse to guess where the missing
/// marker should go and return an error — silently swallowing arbitrary
/// downstream text would be a worse failure mode than asking the human to
/// look.
///
/// The operation is idempotent: `splice(splice(r, c), c) == splice(r, c)`.
pub fn splice(readme: &str, new_content: &str) -> Result<String, Box<dyn Error>> {
    let begin_idx = readme.find(MARKER_BEGIN);
    let end_idx = readme.find(MARKER_END);

    match (begin_idx, end_idx) {
        (Some(b), Some(e)) if b < e => {
            // Replace the region [b..e+|MARKER_END|] with a freshly
            // composed block. We preserve everything before the BEGIN
            // marker and everything after the END marker byte-for-byte.
            let prefix = &readme[..b];
            let suffix = &readme[e + MARKER_END.len()..];
            let block = format!("{MARKER_BEGIN}\n{}\n{MARKER_END}", new_content.trim_end());
            Ok(format!("{prefix}{block}{suffix}"))
        }
        (None, None) => {
            // No markers yet — append a fresh dashboard region.
            let separator = if readme.ends_with("\n\n") || readme.is_empty() {
                ""
            } else if readme.ends_with('\n') {
                "\n"
            } else {
                "\n\n"
            };
            let block = format!(
                "{separator}{MARKER_BEGIN}\n{}\n{MARKER_END}\n",
                new_content.trim_end()
            );
            Ok(format!("{readme}{block}"))
        }
        _ => Err(format!(
            "README has only one of `{MARKER_BEGIN}` / `{MARKER_END}` — \
             refusing to guess the missing boundary. Fix the file by hand."
        )
        .into()),
    }
}

// ===========================================================================
// Rendering
// ===========================================================================

/// Convert a [`DashboardData`] into the markdown body that goes between
/// the dashboard markers.
///
/// The output deliberately omits any field that is empty (no modules → no
/// table; no orphans → no warning section). A dashboard that prints
/// "Orphans: (none)" is noise; a dashboard that says nothing when things
/// are healthy is signal.
pub fn render(data: &DashboardData) -> String {
    let mut out = String::new();

    out.push_str("## Project status\n\n");

    // Provenance line: which commit, which toolchain, which platform.
    // Always rendered. If the working tree was dirty when the dashboard
    // was generated, prepend a loud notice so a reader does not trust
    // numbers attributed to a commit that does not match the disk state.
    if data.dirty {
        out.push_str(
            "> **Working tree was dirty at generation time** — figures \
             below reflect uncommitted changes, not commit \
             `",
        );
        out.push_str(&data.commit);
        out.push_str("`.\n\n");
    }
    out.push_str(&format!(
        "_Generated for commit `{}` on `{}`, toolchain `{}`._\n\n",
        data.commit, data.platform, data.rustc_version
    ));

    // Module LOC table. Sorted by line count descending so the heaviest
    // module is on top — that's where the eyes go first.
    if !data.modules.is_empty() {
        out.push_str("### Modules (LOC)\n\n");
        out.push_str("| Module | LOC |\n|---|---:|\n");
        let total: usize = data.modules.iter().map(|m| m.loc).sum();
        for m in &data.modules {
            out.push_str(&format!("| `{}` | {} |\n", m.name, m.loc));
        }
        out.push_str(&format!("| **total** | **{total}** |\n\n"));
    }

    // Smells. Honest labelling: this is grep, not analysis.
    let s = &data.smells;
    if s.unwrap_count + s.clone_count + s.panic_count + s.unsafe_count > 0 {
        out.push_str("### Hygiene (heuristic grep over `src/`)\n\n");
        out.push_str(&format!(
            "| `.unwrap()` | `.clone()` | `panic!` | `unsafe` |\n\
             |---:|---:|---:|---:|\n\
             | {} | {} | {} | {} |\n\n",
            s.unwrap_count, s.clone_count, s.panic_count, s.unsafe_count
        ));
    }

    // Orphans. Loudly named, one per line, because each one is a silent
    // production bug waiting to happen (a file that the compiler never
    // sees yet still claims attention from readers).
    if !data.orphans.is_empty() {
        out.push_str("### Orphan modules (present on disk, not declared by any `mod`)\n\n");
        for name in &data.orphans {
            out.push_str(&format!("- `{name}`\n"));
        }
        out.push('\n');
    }

    // Test corpus. Counts of test files per directory glob. This is the
    // size of the test set, NOT a pass/fail signal — we explicitly call
    // that out so a reader does not infer "150 tests pass" from "150
    // files exist".
    if !data.test_corpus.is_empty() {
        out.push_str("### Test corpus (file counts; pass/fail not measured here)\n\n");
        out.push_str("| Path | Files |\n|---|---:|\n");
        let mut total = 0usize;
        for e in &data.test_corpus {
            total += e.count;
            out.push_str(&format!("| `{}` | {} |\n", e.glob, e.count));
        }
        out.push_str(&format!("| **total** | **{total}** |\n\n"));
    }

    // Trailing pointer to where behavior data lives. The static dashboard
    // never claims to know whether tests pass; it tells the reader how to
    // find out.
    out.push_str(
        "Behavioral signals (test pass/fail, benchmark throughput) are not \
         part of this static dashboard. Run `./test_dev.sh` for parser \
         coverage and `cargo bench` for performance.\n",
    );

    out
}
