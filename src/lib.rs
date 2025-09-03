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
//! - Supporting both binaries and examples
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
