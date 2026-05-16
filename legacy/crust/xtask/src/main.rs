//! xtask CLI entry point.
//!
//! Subcommands delegate to [`xtask::dashboard`] for the pure logic; the
//! binary's only job is argument parsing, filesystem I/O, and subprocess
//! invocation (`git`, `rustc`, `uname`).

use std::env;
use std::error::Error;
use std::fs;
use std::path::Path;
use std::process::{Command, ExitCode};

use clap::{Parser, Subcommand};

use xtask::dashboard::{
    DashboardData, ModuleStats, SmellCounts, TestCorpusEntry, count_smells,
    find_orphan_modules, render, splice,
};

#[derive(Parser)]
#[command(name = "xtask", about = "Project automation for crust.")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Regenerate the auto-generated region of README.md.
    Dashboard {
        /// Exit non-zero if README.md is not already up to date.
        /// Use this in CI to gate merges on a fresh dashboard.
        #[arg(long)]
        check: bool,

        /// Print the dashboard markdown to stdout; do not touch README.md.
        /// Useful for piping into other tools or for inspecting what
        /// would be written without committing to a write.
        #[arg(long)]
        print: bool,
    },
}

fn main() -> ExitCode {
    match real_main() {
        Ok(code) => code,
        Err(e) => {
            eprintln!("xtask: {e}");
            ExitCode::from(2)
        }
    }
}

fn real_main() -> Result<ExitCode, Box<dyn Error>> {
    let cli = Cli::parse();
    // The xtask crate sits at <workspace_root>/xtask/. We resolve the
    // workspace root from xtask's own manifest dir, which is fixed at
    // compile time and therefore independent of the current working
    // directory (so `cargo xtask` works from any sub-directory).
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("xtask must live one level below the workspace root")?;

    match cli.cmd {
        Cmd::Dashboard { check, print } => run_dashboard(workspace_root, check, print),
    }
}

// ---------------------------------------------------------------------------
// `dashboard` subcommand
// ---------------------------------------------------------------------------

fn run_dashboard(root: &Path, check: bool, print: bool) -> Result<ExitCode, Box<dyn Error>> {
    let src_dir = root.join("src");
    let readme_path = root.join("README.md");

    let data = collect_dashboard_data(root, &src_dir)?;
    let body = render(&data);

    if print {
        print!("{body}");
        return Ok(ExitCode::SUCCESS);
    }

    let old_readme = fs::read_to_string(&readme_path)?;
    let new_readme = splice(&old_readme, &body)?;

    if check {
        if new_readme == old_readme {
            eprintln!("dashboard: README.md is up to date");
            Ok(ExitCode::SUCCESS)
        } else {
            eprintln!(
                "dashboard: README.md is OUT OF DATE. \
                 Run `cargo xtask dashboard` to refresh."
            );
            Ok(ExitCode::from(1))
        }
    } else if new_readme == old_readme {
        eprintln!("dashboard: README.md already current; no changes written");
        Ok(ExitCode::SUCCESS)
    } else {
        fs::write(&readme_path, &new_readme)?;
        eprintln!(
            "dashboard: wrote {} bytes to {}",
            new_readme.len(),
            readme_path.display()
        );
        Ok(ExitCode::SUCCESS)
    }
}

// ---------------------------------------------------------------------------
// Data collection: read the working tree and the environment.
//
// Every function here is a thin adapter from "the world" to one of the
// fields of `DashboardData`. The pure logic lives in `xtask::dashboard`.
// ---------------------------------------------------------------------------

fn collect_dashboard_data(root: &Path, src_dir: &Path) -> Result<DashboardData, Box<dyn Error>> {
    let modules = collect_modules(src_dir)?;
    let smells = collect_smells(src_dir)?;
    let orphans = find_orphan_modules(src_dir)?
        .into_iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    let test_corpus = collect_test_corpus(root)?;

    let rustc_version = command_stdout("rustc", &["-V"])
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "rustc (unknown)".into());

    // Platform string combines what `uname -srm` reports (running kernel)
    // with `env::consts::ARCH` (binary target). These should agree on a
    // sane host; if they disagree, the dashboard reader should investigate.
    let kernel = command_stdout("uname", &["-srm"])
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| format!("{} {}", env::consts::OS, env::consts::ARCH));
    let platform = kernel;

    let commit = command_stdout("git", &["rev-parse", "--short=12", "HEAD"])
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "no-git".into());

    // `git status --porcelain` is empty iff the working tree exactly
    // matches HEAD. Anything else (untracked, modified, staged) is dirty.
    let dirty = command_stdout("git", &["status", "--porcelain"])
        .map(|s| !s.trim().is_empty())
        .unwrap_or(true); // If git failed, assume dirty (the loud failure).

    Ok(DashboardData {
        modules,
        smells,
        orphans,
        test_corpus,
        rustc_version,
        platform,
        commit,
        dirty,
    })
}

/// One row per `.rs` file in `src_dir`. Sorted by descending line count so
/// the biggest module is at the top — that's where the eye lands first.
fn collect_modules(src_dir: &Path) -> Result<Vec<ModuleStats>, Box<dyn Error>> {
    let mut out = Vec::new();
    for entry in fs::read_dir(src_dir)? {
        let path = entry?.path();
        if path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        let content = fs::read_to_string(&path)?;
        out.push(ModuleStats {
            name: path.file_name().unwrap().to_string_lossy().into_owned(),
            loc: content.lines().count(),
        });
    }
    out.sort_by(|a, b| b.loc.cmp(&a.loc).then_with(|| a.name.cmp(&b.name)));
    Ok(out)
}

/// Sum the smell counts across every `.rs` file under `src_dir`.
fn collect_smells(src_dir: &Path) -> Result<SmellCounts, Box<dyn Error>> {
    let mut total = SmellCounts::default();
    for entry in fs::read_dir(src_dir)? {
        let path = entry?.path();
        if path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        let content = fs::read_to_string(&path)?;
        let c = count_smells(&content);
        total.unwrap_count += c.unwrap_count;
        total.clone_count += c.clone_count;
        total.panic_count += c.panic_count;
        total.unsafe_count += c.unsafe_count;
    }
    Ok(total)
}

/// Test corpus is the set of `.c` files this project ships for parser
/// validation, hand-mirrored from the directories iterated by `test_dev.sh`.
///
/// If `test_dev.sh` ever changes which directories it walks, this table
/// must be updated to match — otherwise the dashboard will silently
/// undercount tests. A future refinement would have the dashboard parse
/// `test_dev.sh` directly.
fn collect_test_corpus(root: &Path) -> Result<Vec<TestCorpusEntry>, Box<dyn Error>> {
    const TEST_DIRS: &[&str] = &[
        "test/valid",
        "test/invalid",
        "test/valid/parser",
        "test/valid/cpp",
        "sample_code",
    ];

    let mut out = Vec::new();
    for dir in TEST_DIRS {
        let path = root.join(dir);
        if !path.is_dir() {
            continue;
        }
        let mut count = 0;
        for entry in fs::read_dir(&path)? {
            let p = entry?.path();
            if p.is_file() && p.extension().and_then(|s| s.to_str()) == Some("c") {
                count += 1;
            }
        }
        out.push(TestCorpusEntry {
            glob: format!("{dir}/*.c"),
            count,
        });
    }
    Ok(out)
}

/// Run an external command and return its stdout as a `String`. Returns
/// `Err` if the command fails to spawn, exits non-zero, or produces
/// non-UTF-8 output. Callers fall back to a sentinel string when this
/// errors, so the dashboard can still be produced on systems without
/// `git`, `rustc -V`, or `uname`.
fn command_stdout(cmd: &str, args: &[&str]) -> Result<String, Box<dyn Error>> {
    let out = Command::new(cmd).args(args).output()?;
    if !out.status.success() {
        return Err(format!("{cmd} {args:?} exited with status {}", out.status).into());
    }
    Ok(String::from_utf8(out.stdout)?)
}
