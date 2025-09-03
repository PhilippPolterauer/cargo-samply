//! Command-line interface configuration for cargo-samply.
//!
//! This module defines the CLI structure using `clap` with support for both
//! direct execution (`cargo-samply`) and cargo subcommand usage (`cargo samply`).
//!
//! # Examples
//!
//! ```no_run
//! use cargo_samply::cli::{CargoCli, Config};
//! use clap::Parser;
//!
//! // Parse command-line arguments
//! let CargoCli::Samply(config) = CargoCli::parse();
//! println!("Profile: {}", config.profile);
//! ```

use clap::Parser;

/// The main cargo CLI enum that wraps the samply subcommand.
///
/// This enum is designed to work with cargo's subcommand protocol,
/// allowing the tool to be called as both `cargo samply` and `cargo-samply`.
#[derive(Parser)] // requires `derive` feature
#[command(name = "cargo")]
#[command(bin_name = "cargo")]
#[command(styles = CLAP_STYLING)]
pub enum CargoCli {
    /// The samply subcommand
    Samply(Config),
}

// See also `clap_cargo::style::CLAP_STYLING`
pub const CLAP_STYLING: clap::builder::styling::Styles = clap::builder::styling::Styles::styled()
    .header(clap_cargo::style::HEADER)
    .usage(clap_cargo::style::USAGE)
    .literal(clap_cargo::style::LITERAL)
    .placeholder(clap_cargo::style::PLACEHOLDER)
    .error(clap_cargo::style::ERROR)
    .valid(clap_cargo::style::VALID)
    .invalid(clap_cargo::style::INVALID);

/// Configuration structure for the cargo-samply command.
///
/// This struct contains all the command-line options and arguments
/// that can be passed to cargo-samply.
///
/// # Examples
///
/// ```no_run
/// use cargo_samply::cli::Config;
///
/// let config = Config {
///     args: vec!["--help".to_string()],
///     profile: "samply".to_string(),
///     bin: Some("my-binary".to_string()),
///     example: None,
///     features: vec!["feature1".to_string(), "feature2".to_string()],
///     no_default_features: false,
///     verbose: false,
///     quiet: false,
///     no_samply: false,
/// };
/// ```
#[derive(clap::Args)]
#[command(author, version, about, long_about = None)]
pub struct Config {
    /// Trailing arguments passed to the binary being profiled
    #[arg(name = "TRAILING_ARGUMENTS")]
    pub args: Vec<String>,

    /// Build with the specified profile
    #[arg(short, long, default_value = "samply")]
    pub profile: String,

    /// Binary to run
    #[arg(short, long)]
    pub bin: Option<String>,

    /// Example to run
    #[arg(short, long)]
    pub example: Option<String>,

    /// Build features to enable
    #[arg(short, long)]
    pub features: Vec<String>,

    /// Disable default features
    #[arg(long)]
    pub no_default_features: bool,

    /// Print extra output to help debug problems
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,

    /// Suppress all output except errors
    #[arg(short, long, default_value_t = false)]
    pub quiet: bool,

    /// Disable the automatic samply start
    #[arg(short, long, default_value_t = false)]
    pub no_samply: bool,
}
