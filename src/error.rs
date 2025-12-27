//! Error handling for cargo-samply.
//!
//! This module provides a comprehensive error type using `thiserror` that covers
//! all the various failure modes that can occur during the build and profiling process.
//!
//! # Examples
//!
//! ```no_run
//! use cargo_samply::error::{Error, Result};
//! use std::path::PathBuf;
//!
//! fn example_function() -> Result<()> {
//!     let path = PathBuf::from("nonexistent");
//!     if !path.exists() {
//!         return Err(Error::BinaryNotFound { path });
//!     }
//!     Ok(())
//! }
//! ```

use std::io;
use std::path::PathBuf;
use std::result;
use std::str::Utf8Error;
use thiserror::Error;

/// Comprehensive error type for cargo-samply operations.
///
/// This enum covers all possible error conditions that can occur during
/// the build and profiling process, providing detailed error messages
/// and context where appropriate.
#[derive(Debug, Error)]
pub enum Error {
    /// I/O error with path context for better error reporting
    #[error("{path}: {source}")]
    PathIo { source: io::Error, path: PathBuf },
    /// Generic I/O error
    #[error(transparent)]
    Io(#[from] io::Error),
    /// Logger initialization error
    #[error(transparent)]
    Logger(#[from] log::SetLoggerError),
    /// UTF-8 conversion error
    #[error(transparent)]
    Utf8(#[from] Utf8Error),
    /// TOML deserialization error
    #[error(transparent)]
    TomlDeserialization(#[from] toml::de::Error),
    /// Cargo.toml manifest parsing error
    #[error(transparent)]
    TomlManifest(#[from] cargo_toml::Error),
    /// Target-selection flags (bin/example/bench/test) are mutually exclusive
    #[error("Target selection flags (--bin, --example, --bench, --test) are mutually exclusive")]
    MultipleTargetsFlagsSpecified,
    /// Cargo build process failed
    #[error("Build failed")]
    CargoBuildFailed,
    /// No binary targets found in Cargo.toml
    #[error("No binary found in 'Cargo.toml'")]
    NoBinaryFound,
    /// Multiple binaries found but no default specified
    #[error("The binary to run can't be determined. Use the `--bin` option to specify a binary, or the `default-run` manifest key.{suggestions}")]
    BinaryToRunNotDetermined { suggestions: String },
    /// Failed to locate the cargo project
    #[error("Failed to locate project")]
    CargoLocateProjectFailed,
    /// Built binary not found in target directory
    #[error("Binary not found: {path}")]
    BinaryNotFound { path: PathBuf },
    /// Package not found in workspace
    #[error("Package '{name}' not found in workspace")]
    PackageNotFound { name: String },
    /// Samply binary not installed or not in PATH
    #[error("samply is not installed or not in PATH")]
    SamplyNotFound,
}

/// Alias for a `Result` with the error type `cargo_samply::Error`.
///
/// This type alias simplifies function signatures throughout the codebase
/// by providing a default error type.
pub type Result<T> = result::Result<T, Error>;

/// Extension trait for `io::Result` to add path context to I/O errors.
///
/// This trait provides a convenient way to add file path information
/// to I/O errors, making debugging easier.
///
/// # Examples
///
/// ```no_run
/// use cargo_samply::error::{IOResultExt, Result};
/// use std::fs;
/// use std::path::Path;
///
/// fn read_file(path: &Path) -> Result<String> {
///     fs::read_to_string(path).path_ctx(path)
/// }
/// ```
pub trait IOResultExt<T> {
    /// Add path context to an I/O result
    fn path_ctx<P: Into<PathBuf>>(self, path: P) -> Result<T>;
}

impl<T> IOResultExt<T> for io::Result<T> {
    fn path_ctx<P: Into<PathBuf>>(self, path: P) -> Result<T> {
        self.map_err(|source| Error::PathIo {
            source,
            path: path.into(),
        })
    }
}
