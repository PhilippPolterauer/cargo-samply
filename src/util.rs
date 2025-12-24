//! Utility functions for cargo-samply.
//!
//! This module contains helper functions for:
//! - Locating cargo projects
//! - Managing the samply profile in Cargo.toml
//! - Determining which binary to run
//! - Command execution with logging
//!
//! # Examples
//!
//! ```no_run
//! use cargo_samply::util::{locate_project, guess_bin, ensure_samply_profile};
//! use cargo_samply::error::Result;
//!
//! fn example() -> Result<()> {
//!     let cargo_toml = locate_project()?;
//!     ensure_samply_profile(&cargo_toml)?;
//!     let binary = guess_bin(&cargo_toml)?;
//!     println!("Will run binary: {}", binary);
//!     Ok(())
//! }
//! ```

use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    process::{Command, ExitStatus},
    str::{from_utf8, FromStr},
};

use crate::error::{self, IOResultExt};
use cargo_metadata::MetadataCommand;
use log::{debug, info};

#[derive(Debug)]
pub struct WorkspaceMetadata {
    pub binaries: Vec<String>,
    pub examples: Vec<String>,
    pub benches: Vec<String>,
}

/// Locates the cargo project by running `cargo locate-project`.
///
/// This function uses cargo's built-in project location functionality
/// to find the path to the project's `Cargo.toml` file.
///
/// # Returns
///
/// - `Ok(PathBuf)` - Path to the `Cargo.toml` file
/// - `Err(Error::CargoLocateProjectFailed)` - If cargo command fails
///
/// # Examples
///
/// ```no_run
/// use cargo_samply::util::locate_project;
///
/// let cargo_toml = locate_project()?;
/// println!("Found Cargo.toml at: {}", cargo_toml.display());
/// # Ok::<(), cargo_samply::error::Error>(())
/// ```
pub fn locate_project() -> error::Result<PathBuf> {
    let output = Command::new("cargo")
        .args(vec![
            "locate-project",
            "--workspace",
            "--message-format",
            "plain",
        ])
        .log()
        .output()?;
    if !output.status.success() {
        return Err(error::Error::CargoLocateProjectFailed);
    }
    Ok(PathBuf::from(from_utf8(&output.stdout)?.trim()))
}

/// The samply profile configuration that gets added to Cargo.toml.
///
/// This profile inherits from the release profile but enables debug symbols
/// for accurate profiling information.
const SAMPLY_PROFILE: &str = "
[profile.samply]
inherits = \"release\"
debug = true
";

/// Ensures that the samply profile exists in the given Cargo.toml file.
///
/// This function checks if a `[profile.samply]` section exists in the Cargo.toml.
/// If it doesn't exist, it appends the profile configuration to the file.
///
/// # Arguments
///
/// * `cargo_toml` - Path to the Cargo.toml file
///
/// # Returns
///
/// - `Ok(())` - Profile exists or was successfully added
/// - `Err(Error)` - If file operations fail
///
/// # Examples
///
/// ```no_run
/// use cargo_samply::util::ensure_samply_profile;
/// use std::path::Path;
///
/// let cargo_toml = Path::new("Cargo.toml");
/// ensure_samply_profile(cargo_toml)?;
/// # Ok::<(), cargo_samply::error::Error>(())
/// ```
pub fn ensure_samply_profile(cargo_toml: &Path) -> error::Result<()> {
    let cargo_toml_content: String = fs::read_to_string(cargo_toml).path_ctx(cargo_toml)?;
    let manifest = toml::Table::from_str(&cargo_toml_content)?;
    let profile_samply = manifest
        .get("profile")
        .and_then(|p| p.as_table())
        .and_then(|p| p.get("samply"));

    if profile_samply.is_none() {
        let mut f = OpenOptions::new()
            .append(true)
            .open(cargo_toml)
            .path_ctx(cargo_toml)?;
        f.write(SAMPLY_PROFILE.as_bytes()).path_ctx(cargo_toml)?;
        info!("'samply' profile was added to 'Cargo.toml'");
    }
    Ok(())
}

/// Gets workspace metadata including all available binaries and examples.
///
/// This function uses `cargo_metadata` to collect information about all
/// binaries and examples available in the workspace.
///
/// # Arguments
///
/// * `cargo_toml` - Path to the Cargo.toml file to determine the working directory
///
/// # Returns
///
/// - `Ok(WorkspaceMetadata)` - Metadata containing available binaries and examples
/// - `Err(Error)` - If cargo metadata command fails
pub fn get_workspace_metadata_from(cargo_toml: &Path) -> error::Result<WorkspaceMetadata> {
    let work_dir = cargo_toml.parent().unwrap_or_else(|| Path::new("."));

    let metadata = MetadataCommand::new()
        .current_dir(work_dir)
        .no_deps()
        .exec()
        .map_err(|e| error::Error::Io(std::io::Error::other(e)))?;

    let mut binaries = Vec::new();
    let mut examples = Vec::new();
    let mut benches = Vec::new();

    for package in metadata.packages {
        for target in package.targets {
            if target.is_bin() {
                if !binaries.contains(&target.name) {
                    binaries.push(target.name);
                }
            } else if target.is_example() {
                if !examples.contains(&target.name) {
                    examples.push(target.name);
                }
            } else if target.kind.contains(&cargo_metadata::TargetKind::Bench)
                && !benches.contains(&target.name)
            {
                benches.push(target.name);
            }
        }
    }

    binaries.sort();
    examples.sort();
    benches.sort();

    Ok(WorkspaceMetadata {
        binaries,
        examples,
        benches,
    })
}

/// Resolves the requested bench target name if it exists in the local
/// manifest or workspace metadata. Matching is exact; no suffix munging.
pub fn resolve_bench_target_name(cargo_toml: &Path, requested: &str) -> error::Result<String> {
    let manifest = cargo_toml::Manifest::from_path(cargo_toml)?;
    let local_benches: Vec<String> = manifest
        .bench
        .iter()
        .filter_map(|bench| bench.name.clone())
        .collect();

    if let Some(found) = select_matching_bench(requested, &local_benches) {
        return Ok(found);
    }

    let workspace_metadata = get_workspace_metadata_from(cargo_toml)?;
    if let Some(found) = select_matching_bench(requested, &workspace_metadata.benches) {
        return Ok(found);
    }

    Ok(requested.to_string())
}

fn select_matching_bench(requested: &str, benches: &[String]) -> Option<String> {
    benches
        .iter()
        .find(|candidate| candidate.as_str() == requested)
        .cloned()
}

/// Determines which binary to run based on the Cargo.toml configuration.
///
/// This function uses the following priority order:
/// 1. If `default-run` is specified in `[package]`, use that binary
/// 2. If there's exactly one binary in the local manifest, use that binary
/// 3. If there are no binaries in local manifest, try workspace metadata
/// 4. If there are multiple binaries, return `BinaryToRunNotDetermined` with suggestions
///
/// # Arguments
///
/// * `cargo_toml` - Path to the Cargo.toml file
///
/// # Returns
///
/// - `Ok(String)` - Name of the binary to run
/// - `Err(Error::NoBinaryFound)` - No binary targets found
/// - `Err(Error::BinaryToRunNotDetermined)` - Multiple binaries, includes suggestions
///
/// # Examples
///
/// ```no_run
/// use cargo_samply::util::guess_bin;
/// use std::path::Path;
///
/// let cargo_toml = Path::new("Cargo.toml");
/// let binary_name = guess_bin(cargo_toml)?;
/// println!("Will run binary: {}", binary_name);
/// # Ok::<(), cargo_samply::error::Error>(())
/// ```
pub fn guess_bin(cargo_toml: &Path) -> error::Result<String> {
    // First try the local manifest for default-run
    let manifest = cargo_toml::Manifest::from_path(cargo_toml)?;
    let default_run = manifest.package.and_then(|p| p.default_run);
    if let Some(bin) = default_run {
        return Ok(bin);
    }

    // Check local manifest binaries first
    if manifest.bin.len() == 1 {
        if let Some(name) = manifest.bin.first().and_then(|b| b.name.as_ref()) {
            return Ok(name.clone());
        }
    }

    // If local manifest has multiple binaries, collect them for suggestions
    let local_binaries: Vec<String> = manifest
        .bin
        .iter()
        .filter_map(|b| b.name.as_ref())
        .cloned()
        .collect();

    let local_examples: Vec<String> = manifest
        .example
        .iter()
        .filter_map(|e| e.name.as_ref())
        .cloned()
        .collect();

    // If we have local binaries/examples, use them for suggestions
    if !local_binaries.is_empty() || !local_examples.is_empty() {
        return create_suggestions_error(local_binaries, local_examples);
    }

    // Fall back to workspace metadata for complex workspace scenarios
    let workspace_metadata = get_workspace_metadata_from(cargo_toml).unwrap_or_else(|_| {
        // If cargo metadata fails, return empty metadata
        WorkspaceMetadata {
            binaries: Vec::new(),
            examples: Vec::new(),
            benches: Vec::new(),
        }
    });

    if workspace_metadata.binaries.is_empty() {
        return Err(error::Error::NoBinaryFound);
    }

    if workspace_metadata.binaries.len() == 1 {
        return Ok(workspace_metadata.binaries[0].clone());
    }

    create_suggestions_error(workspace_metadata.binaries, workspace_metadata.examples)
}

fn create_suggestions_error(binaries: Vec<String>, examples: Vec<String>) -> error::Result<String> {
    let mut suggestions = Vec::new();

    if !binaries.is_empty() {
        suggestions.push("\n\nAvailable binaries:".to_string());
        for bin in &binaries {
            suggestions.push(format!("  {}: cargo samply --bin {}", bin, bin));
        }
    }

    if !examples.is_empty() {
        suggestions.push("\n\nAvailable examples:".to_string());
        for example in &examples {
            suggestions.push(format!("  {}: cargo samply --example {}", example, example));
        }
    }

    let suggestions_text = suggestions.join("\n");

    Err(error::Error::BinaryToRunNotDetermined {
        suggestions: suggestions_text,
    })
}

/// Extension trait for `Command` that adds logging and convenience methods.
///
/// This trait provides additional functionality for running commands
/// with automatic logging in debug mode and error handling.
///
/// # Examples
///
/// ```no_run
/// use cargo_samply::util::CommandExt;
/// use std::process::Command;
///
/// let exit_status = Command::new("cargo")
///     .args(&["build", "--release"])
///     .call()?;
///
/// if exit_status.success() {
///     println!("Build succeeded!");
/// }
/// # Ok::<(), cargo_samply::error::Error>(())
/// ```
pub trait CommandExt {
    /// Execute the command and return the exit status.
    ///
    /// This method automatically logs the command and its arguments
    /// in debug mode before execution.
    fn call(&mut self) -> error::Result<ExitStatus>;

    /// Log the command and its arguments in debug mode.
    ///
    /// This method is called automatically by `call()` but can also
    /// be used standalone for debugging purposes.
    fn log(&mut self) -> &mut Command;
}

impl CommandExt for Command {
    fn call(&mut self) -> error::Result<ExitStatus> {
        self.log();
        Ok(self.spawn()?.wait()?)
    }
    fn log(&mut self) -> &mut Command {
        debug!(
            "running {:?} with args: {:?}",
            self.get_program(),
            self.get_args().collect::<Vec<&std::ffi::OsStr>>()
        );
        self
    }
}

/// Gets the Rust sysroot path by running `rustc --print sysroot`.
///
/// # Returns
///
/// - `Ok(PathBuf)` - Path to the Rust sysroot
/// - `Err(Error)` - If rustc command fails or output is invalid
fn get_rust_sysroot() -> error::Result<PathBuf> {
    let output = Command::new("rustc")
        .arg("--print")
        .arg("sysroot")
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(error::Error::Io(io::Error::other(format!(
            "Failed to get Rust sysroot: {}",
            stderr.trim()
        ))));
    }

    let sysroot = from_utf8(&output.stdout)?.trim();
    Ok(PathBuf::from(sysroot))
}

/// Configures a command so that both Rust toolchain libs and target `deps/` shared libraries
/// can be found at runtime.
///
/// This mirrors what `cargo run` effectively provides for crates using dynamic linking
/// (e.g. Bevy's `dynamic_linking` feature, which produces a `bevy_dylib` shared library
/// under `target/<profile>/deps`).
pub fn configure_library_path_for_binary(
    cmd: &mut Command,
    bin_path: &std::path::Path,
) -> error::Result<()> {
    let mut extra_paths = Vec::new();
    if let Some(bin_dir) = bin_path.parent() {
        if bin_dir
            .file_name()
            .is_some_and(|name| name == std::ffi::OsStr::new("deps"))
        {
            extra_paths.push(bin_dir.to_path_buf());
        } else {
            extra_paths.push(bin_dir.join("deps"));
        }
    }
    configure_library_path_impl(cmd, &extra_paths)
}

fn configure_library_path_impl(
    cmd: &mut Command,
    extra_paths: &[std::path::PathBuf],
) -> error::Result<()> {
    // Allow disabling sysroot injection for testing purposes
    if std::env::var("CARGO_SAMPLY_NO_SYSROOT_INJECTION").is_ok() {
        debug!("Skipping sysroot injection (CARGO_SAMPLY_NO_SYSROOT_INJECTION is set)");
        return Ok(());
    }

    let sysroot = get_rust_sysroot()?;

    // Determine the correct environment variable based on the platform
    let (env_var_name, separator) = if cfg!(target_os = "macos") {
        ("DYLD_LIBRARY_PATH", ":")
    } else if cfg!(target_os = "windows") {
        ("PATH", ";")
    } else {
        // Linux and other Unix-like systems
        ("LD_LIBRARY_PATH", ":")
    };

    // Get the current value of the environment variable, preferring an explicit
    // value already configured on `cmd`.
    let current_val = cmd
        .get_envs()
        .find_map(|(k, v)| {
            (k == std::ffi::OsStr::new(env_var_name))
                .then(|| v.map(|v| v.to_string_lossy().into_owned()))
                .flatten()
        })
        .or_else(|| std::env::var_os(env_var_name).map(|s| s.to_string_lossy().into_owned()))
        .unwrap_or_default();

    // Build the library paths. We need to include both the general lib directory
    // and the target-specific rustlib directory (like cargo does).
    // The target triple is detected from the host system.
    let lib_path = sysroot.join("lib");
    let target_triple = std::env::var("TARGET")
        .or_else(|_| std::env::var("CARGO_BUILD_TARGET"))
        .unwrap_or_else(|_| {
            // Default to the current host target triple
            if cfg!(target_os = "macos") && cfg!(target_arch = "x86_64") {
                "x86_64-apple-darwin".to_string()
            } else if cfg!(target_os = "macos") && cfg!(target_arch = "aarch64") {
                "aarch64-apple-darwin".to_string()
            } else if cfg!(target_os = "linux") && cfg!(target_arch = "x86_64") {
                "x86_64-unknown-linux-gnu".to_string()
            } else if cfg!(target_os = "linux") && cfg!(target_arch = "aarch64") {
                "aarch64-unknown-linux-gnu".to_string()
            } else if cfg!(target_os = "windows") && cfg!(target_arch = "x86_64") {
                "x86_64-pc-windows-msvc".to_string()
            } else if cfg!(target_os = "windows") && cfg!(target_arch = "aarch64") {
                "aarch64-pc-windows-msvc".to_string()
            } else {
                // Fallback - try to get from rustc
                "unknown".to_string()
            }
        });

    let target_lib_path = sysroot
        .join("lib")
        .join("rustlib")
        .join(&target_triple)
        .join("lib");

    let mut parts: Vec<String> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();

    for p in extra_paths {
        let s = p.to_string_lossy().into_owned();
        if !s.is_empty() && seen.insert(s.clone()) {
            parts.push(s);
        }
    }

    for p in [&target_lib_path, &lib_path] {
        let s = p.to_string_lossy().into_owned();
        if !s.is_empty() && seen.insert(s.clone()) {
            parts.push(s);
        }
    }

    if !current_val.is_empty() {
        for seg in current_val.split(separator) {
            let seg = seg.trim();
            if !seg.is_empty() && seen.insert(seg.to_string()) {
                parts.push(seg.to_string());
            }
        }
    }

    let new_val = parts.join(separator);

    debug!("Setting {} to: {}", env_var_name, new_val);

    cmd.env(env_var_name, new_val);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::{Mutex, OnceLock};
    use tempfile::TempDir;

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
    }

    #[test]
    fn test_ensure_samply_profile_adds_profile() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_toml_path = temp_dir.path().join("Cargo.toml");
        let initial_content = r#"
[package]
name = "test"
version = "0.1.0"
"#;
        fs::write(&cargo_toml_path, initial_content).unwrap();

        ensure_samply_profile(&cargo_toml_path).unwrap();

        let content = fs::read_to_string(&cargo_toml_path).unwrap();
        assert!(content.contains("[profile.samply]"));
        assert!(content.contains("inherits = \"release\""));
        assert!(content.contains("debug = true"));
    }

    #[test]
    fn test_ensure_samply_profile_already_exists() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_toml_path = temp_dir.path().join("Cargo.toml");
        let initial_content = r#"
[package]
name = "test"
version = "0.1.0"

[profile.samply]
inherits = "release"
debug = true
"#;
        fs::write(&cargo_toml_path, initial_content).unwrap();
        let original_content = fs::read_to_string(&cargo_toml_path).unwrap();
        ensure_samply_profile(&cargo_toml_path).unwrap();
        let new_content = fs::read_to_string(&cargo_toml_path).unwrap();

        assert_eq!(original_content, new_content); // Should not change
    }

    #[test]
    fn test_guess_bin_with_default_run() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_toml_path = temp_dir.path().join("Cargo.toml");
        let content = r#"
[package]
name = "test"
version = "0.1.0"
default-run = "mybin"

[[bin]]
name = "mybin"
path = "src/main.rs"
"#;
        fs::write(&cargo_toml_path, content).unwrap();

        let bin = guess_bin(&cargo_toml_path).unwrap();
        assert_eq!(bin, "mybin");
    }

    #[test]
    fn test_guess_bin_single_bin() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_toml_path = temp_dir.path().join("Cargo.toml");
        let content = r#"
[package]
name = "test"
version = "0.1.0"

[[bin]]
name = "single"
path = "src/main.rs"
"#;
        fs::write(&cargo_toml_path, content).unwrap();

        let bin = guess_bin(&cargo_toml_path).unwrap();
        assert_eq!(bin, "single");
    }

    #[test]
    fn test_guess_bin_multiple_bins_no_default() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_toml_path = temp_dir.path().join("Cargo.toml");
        let content = r#"
[package]
name = "test"
version = "0.1.0"

[[bin]]
name = "first"
path = "src/first.rs"

[[bin]]
name = "second"
path = "src/second.rs"
"#;
        fs::write(&cargo_toml_path, content).unwrap();

        let result = guess_bin(&cargo_toml_path);
        assert!(result.is_err());
        if let Err(error::Error::BinaryToRunNotDetermined { suggestions: _ }) = result {
            // Correct
        } else {
            panic!("Expected BinaryToRunNotDetermined with suggestions");
        }
    }

    #[test]
    fn test_guess_bin_no_bins() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_toml_path = temp_dir.path().join("Cargo.toml");
        let content = r#"
[package]
name = "test"
version = "0.1.0"
"#;
        fs::write(&cargo_toml_path, content).unwrap();

        let result = guess_bin(&cargo_toml_path);
        assert!(result.is_err());
        if let Err(error::Error::NoBinaryFound) = result {
            // Correct
        } else {
            panic!("Expected NoBinaryFound, got: {:?}", result);
        }
    }

    #[test]
    fn test_get_rust_sysroot_returns_valid_path() {
        // Test that get_rust_sysroot returns a valid path
        let sysroot = get_rust_sysroot().expect("Failed to get Rust sysroot");

        // Verify the path exists
        assert!(sysroot.exists(), "Sysroot path should exist: {:?}", sysroot);

        // Verify it's a directory
        assert!(
            sysroot.is_dir(),
            "Sysroot should be a directory: {:?}",
            sysroot
        );

        // Verify it has a lib subdirectory (which we need for dynamic linking)
        let lib_dir = sysroot.join("lib");
        assert!(
            lib_dir.exists(),
            "Sysroot should have a lib directory: {:?}",
            lib_dir
        );
    }

    #[test]
    fn test_configure_library_path_sets_env_var() {
        let _env_guard = env_lock();
        // Test that sysroot library path injection sets the appropriate environment variable
        let mut cmd = Command::new("echo");

        super::configure_library_path_impl(&mut cmd, &[])
            .expect("Failed to configure library path");

        // Get the environment variables from the command
        let env_vars: std::collections::HashMap<_, _> = cmd
            .get_envs()
            .filter_map(|(k, v)| {
                v.map(|v| {
                    (
                        k.to_string_lossy().to_string(),
                        v.to_string_lossy().to_string(),
                    )
                })
            })
            .collect();

        // Check that the correct environment variable is set based on platform
        let expected_env_var = if cfg!(target_os = "macos") {
            "DYLD_LIBRARY_PATH"
        } else if cfg!(target_os = "windows") {
            "PATH"
        } else {
            "LD_LIBRARY_PATH"
        };

        assert!(
            env_vars.contains_key(expected_env_var),
            "Expected {} to be set, but it was not. Environment: {:?}",
            expected_env_var,
            env_vars
        );

        // Verify the path contains the sysroot lib directory
        let sysroot = get_rust_sysroot().expect("Failed to get Rust sysroot");
        let expected_lib_path = sysroot.join("lib");
        let env_value = env_vars
            .get(expected_env_var)
            .expect("Env var should exist");

        assert!(
            env_value.contains(&expected_lib_path.display().to_string()),
            "Environment variable {} should contain {}, but got: {}",
            expected_env_var,
            expected_lib_path.display(),
            env_value
        );
    }

    #[test]
    fn test_configure_library_path_preserves_existing_path() {
        let _env_guard = env_lock();
        // Test that sysroot library path injection preserves existing paths
        let env_var_name = if cfg!(target_os = "macos") {
            "DYLD_LIBRARY_PATH"
        } else if cfg!(target_os = "windows") {
            "PATH"
        } else {
            "LD_LIBRARY_PATH"
        };

        let existing_path = "/some/existing/path";
        std::env::set_var(env_var_name, existing_path);

        let mut cmd = Command::new("echo");
        super::configure_library_path_impl(&mut cmd, &[])
            .expect("Failed to configure library path");

        // get_envs() returns explicitly set environment variables on the command
        // We need to check if the value was set
        let mut found = false;
        let mut env_value = String::new();
        for (key, value) in cmd.get_envs() {
            if key.to_string_lossy() == env_var_name {
                if let Some(v) = value {
                    found = true;
                    env_value = v.to_string_lossy().to_string();
                    break;
                }
            }
        }

        assert!(
            found,
            "Environment variable {} should be set on command",
            env_var_name
        );

        // Verify existing path is preserved
        assert!(
            env_value.contains(existing_path),
            "Existing path should be preserved. Expected to find '{}' in '{}'",
            existing_path,
            env_value
        );

        // Clean up
        std::env::remove_var(env_var_name);
    }

    #[test]
    fn test_configure_library_path_can_be_disabled() {
        let _env_guard = env_lock();
        // Test that sysroot injection can be disabled via environment variable
        std::env::set_var("CARGO_SAMPLY_NO_SYSROOT_INJECTION", "1");

        let mut cmd = Command::new("echo");
        super::configure_library_path_impl(&mut cmd, &[])
            .expect("Should succeed even when disabled");

        // Get the environment variables from the command
        let env_vars: std::collections::HashMap<_, _> = cmd
            .get_envs()
            .filter_map(|(k, v)| {
                v.map(|v| {
                    (
                        k.to_string_lossy().to_string(),
                        v.to_string_lossy().to_string(),
                    )
                })
            })
            .collect();

        // Verify that library path was NOT set
        let env_var_name = if cfg!(target_os = "macos") {
            "DYLD_LIBRARY_PATH"
        } else if cfg!(target_os = "windows") {
            "PATH"
        } else {
            "LD_LIBRARY_PATH"
        };

        assert!(
            !env_vars.contains_key(env_var_name),
            "Expected {} to NOT be set when injection is disabled, but it was: {:?}",
            env_var_name,
            env_vars
        );

        // Clean up
        std::env::remove_var("CARGO_SAMPLY_NO_SYSROOT_INJECTION");
    }
}
