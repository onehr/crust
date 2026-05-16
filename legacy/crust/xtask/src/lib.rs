//! xtask: project automation for crust.
//!
//! This crate ships one binary (`xtask`) that the workspace exposes as
//! `cargo xtask <subcommand>`. The library half hosts the pure logic so
//! it can be exercised by integration tests under `xtask/tests/`.
//!
//! Design constraint: every signal the dashboard renders must be derivable
//! from local, deterministic inputs (the working tree, `rustc -vV`,
//! `uname`, `git rev-parse HEAD`). Anything requiring a network call, a
//! long-running benchmark, or non-determinism belongs in a separate
//! `--full` mode (out of scope for v1).

pub mod dashboard;
