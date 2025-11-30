//! # cargo-samply
//!
//! A cargo subcommand to automate the process of running [samply](https://github.com/mstange/samply)
//! for profiling Rust project binaries.
//!
//! ## Overview
//!
//! `cargo-samply` simplifies the workflow of profiling Rust applications by:
//! - Automatically building your project with debug symbols
//! - Managing the `samply` profiling profile in `Cargo.toml`
//! - Running `samply` with the correct binary path
//! - Supporting binaries, examples, and benches (Criterion or otherwise)
//! - Providing flexible feature and profile selection
//!
//! ## Installation
//!
//! ```bash
//! cargo install cargo-samply
//! ```
//!
//! You also need to install `samply`:
//! ```bash
//! cargo install samply
//! ```
//!
//! ## Usage
//!
//! ### Basic Usage
//!
//! Profile the default binary:
//! ```bash
//! cargo samply
//! ```
//!
//! Profile a specific binary:
//! ```bash
//! cargo samply --bin my-binary
//! ```
//!
//! Profile an example:
//! ```bash
//! cargo samply --example my-example
//! ```
//!
//! Profile a benchmark target:
//! ```bash
//! cargo samply --bench throughput -- --sample-size 10
//! ```
//!
//! Bench targets can be referenced with or without the trailing `_bench`
//! or `-bench` suffix. For example, `cargo samply --bench gather_rows` will
//! automatically pick the `gather_rows_bench` target.
//!
//! ### Advanced Options
//!
//! Use a different profile:
//! ```bash
//! cargo samply --profile release
//! ```
//!
//! Enable specific features:
//! ```bash
//! cargo samply --features feature1,feature2
//! # or
//! cargo samply --features feature1 --features feature2
//! ```
//!
//! Run without samply (just execute the binary):
//! ```bash
//! cargo samply --no-samply -- arg1 arg2
//! ```
//!
//! Quiet mode (suppress output):
//! ```bash
//! cargo samply --quiet
//! ```
//!
//! Verbose mode (debug output):
//! ```bash
//! cargo samply --verbose
//! ```
//!
//! ### Passing Arguments to the Binary
//!
//! You can pass arguments to the binary being profiled by using `--` to separate
//! cargo-samply options from binary arguments:
//!
//! ```bash
//! # Pass arguments to the default binary
//! cargo samply -- arg1 arg2 --flag value
//! ```
//!
//! ```bash
//! # Pass arguments to a specific binary
//! cargo samply --bin my-binary -- --input file.txt --verbose
//! ```
//!
//! ```bash
//! # Pass arguments to an example
//! cargo samply --example my-example -- --config config.json
//! ```
//!
//! When using `--no-samply` (to just run the binary without profiling),
//! arguments are passed through directly:
//!
//! ```bash
//! # Run binary with arguments but without profiling
//! cargo samply --no-samply --bin my-binary -- --debug --port 8080
//! ```
//!
//! **Note**: All arguments after `--` are passed directly to your binary,
//! so you can use any command-line arguments your binary supports.
//!
//! ## How It Works
//!
//! 1. **Profile Management**: Automatically adds a `samply` profile to your `Cargo.toml` if it doesn't exist
//! 2. **Build**: Compiles your project with the specified profile
//! 3. **Binary Resolution**: Determines which binary to run based on `--bin`, `--example`, or `default-run`
//! 4. **Profiling**: Launches `samply record` with the built binary
//!
//! ## The `samply` Profile
//!
//! The tool automatically adds this profile to your `Cargo.toml`:
//!
//! ```toml
//! [profile.samply]
//! inherits = "release"
//! debug = true
//! ```
//!
//! This provides optimized code with debug symbols for accurate profiling.

pub mod cli;
pub mod error;
pub mod util;

pub use cli::{CargoCli, Config};
pub use error::{Error, Result};
