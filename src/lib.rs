//! # cargo-samply
//!
//! Profile Rust code with [`samply`](https://github.com/mstange/samply), without
//! remembering the build/run ceremony.
//!
//! This crate provides both:
//! - a Cargo subcommand (`cargo samply` / `cargo-samply`)
//! - a small Rust API surface (mainly for CLI parsing / helpers)
//!
//! > Note: This project is **not affiliated** with upstream `samply`.
//!
//! ## Installation
//!
//! ```console
//! $ cargo install cargo-samply
//! $ cargo install samply
//! ```
//!
//! ## Usage
//!
//! Profile the default binary target:
//!
//! ```console
//! $ cargo samply
//! ```
//!
//! Select a specific target:
//!
//! ```console
//! $ cargo samply --bin my-binary
//! $ cargo samply --example my-example
//! $ cargo samply --bench throughput -- --sample-size 10
//! $ cargo samply --test integration_suite
//! ```
//!
//! Bench targets must be referenced using their exact Cargo target names (no
//! suffix rewriting / aliasing).
//!
//! ### Passing arguments
//!
//! Arguments after `--` are passed to the program being profiled:
//!
//! ```console
//! $ cargo samply --bin my-binary -- --input file.txt --verbose
//! ```
//!
//! You can also pass arguments directly to `samply` itself:
//!
//! ```console
//! $ cargo samply --samply-args "--rate 2000" --bin my-binary
//! ```
//!
//! ### Workspaces
//!
//! In a workspace, you can pick the package to profile:
//!
//! ```console
//! $ cargo samply -p my-package --bin my-binary
//! ```
//!
//! ### Bench flag injection
//!
//! By default, when profiling a benchmark via `--bench <name>`, `cargo-samply`
//! will run the final benchmark binary with `--bench` (mirroring `cargo bench`).
//! You can customize this for non-Criterion harnesses:
//!
//! ```console
//! $ cargo samply --bench throughput --bench-flag=--my-custom-flag
//! $ cargo samply --bench throughput --bench-flag=none
//! ```
//!
//! ### Dry-run and target listing
//!
//! `--dry-run` prints the `cargo build` and final execution command without
//! running them. The output is intended to be copy-pasteable in a shell.
//!
//! ```console
//! $ cargo samply --dry-run --bin my-binary -- --arg value
//! ```
//!
//! `--list-targets` prints discovered targets and exits:
//!
//! ```console
//! $ cargo samply --list-targets
//! ```
//!
//! ## Environment variables
//!
//! - `CARGO_SAMPLY_SAMPLY_PATH`: override the path to the `samply` binary.
//! - `CARGO_SAMPLY_NO_PROFILE_INJECT`: disable automatic modification of
//!   `Cargo.toml` (equivalent to `--no-profile-inject`).
//! - `CARGO_SAMPLY_NO_SYSROOT_INJECTION`: disable automatic injection of Rust
//!   sysroot library paths into the runtime loader path.
//!   (Linux: `LD_LIBRARY_PATH`, macOS: `DYLD_LIBRARY_PATH`, Windows: `PATH`).
//!
//! ## How it works (high level)
//!
//! 1. Locates the Cargo project (`cargo locate-project`).
//! 2. Ensures a `[profile.samply]` exists (unless disabled).
//! 3. Builds the selected target with `cargo build`.
//! 4. Resolves the produced artifact path from Cargo metadata/messages.
//! 5. Optionally configures runtime library paths (including Rust sysroot) so
//!    binaries with dynamic Rust dependencies run reliably.
//! 6. Runs either the binary directly (`--no-samply`) or under
//!    `samply record -- <artifact> ...`.
//!
//! ## The `samply` Cargo profile
//!
//! When profile injection is enabled, `cargo-samply` ensures your manifest
//! contains:
//!
//! ```toml
//! [profile.samply]
//! inherits = "release"
//! debug = true
//! ```

pub mod cli;
pub mod error;
pub mod util;

pub use cli::{CargoCli, Config};
pub use error::{Error, Result};
