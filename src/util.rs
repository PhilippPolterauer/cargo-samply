//! Utility functions for cargo-samply.
//!
//! This module contains helper functions for:
//! - Locating cargo projects
//! - Managing the samply profile in Cargo.toml
//! - Determining which binary to run
//! - Command execution with logging

use std::{
    collections::HashSet,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process::{Command, ExitStatus},
    str::{from_utf8, FromStr},
};

use crate::error::{self, IOResultExt};
use cargo_metadata::MetadataCommand;
use log::{debug, info};

/// Metadata about a Cargo workspace, including available targets.
///
/// This struct contains lists of all binaries, examples, benchmarks,
/// and tests discovered in the workspace.
#[derive(Debug)]
pub struct WorkspaceMetadata {
    /// Names of all binary targets in the workspace
    pub binaries: Vec<String>,
    /// Names of all example targets in the workspace
    pub examples: Vec<String>,
    /// Names of all benchmark targets in the workspace
    pub benches: Vec<String>,
    /// Names of all test targets in the workspace
    pub tests: Vec<String>,
    /// Path to the workspace root directory
    pub workspace_root: PathBuf,
}

/// Locates the cargo project by running `cargo locate-project`.
pub fn locate_project() -> error::Result<PathBuf> {
    let output = Command::new("cargo")
        .args(vec!["locate-project", "--message-format", "plain"])
        .log()
        .output()?;
    if !output.status.success() {
        return Err(error::Error::CargoLocateProjectFailed);
    }
    Ok(PathBuf::from(from_utf8(&output.stdout)?.trim()))
}

/// The samply profile configuration that gets added to Cargo.toml.
const SAMPLY_PROFILE: &str = "
[profile.samply]
inherits = \"release\"
debug = true
";

/// Reads Cargo.toml and returns whether the samply profile exists.
fn has_samply_profile_in_manifest(cargo_toml: &Path) -> error::Result<bool> {
    let cargo_toml_content: String = fs::read_to_string(cargo_toml).path_ctx(cargo_toml)?;
    let manifest = toml::Table::from_str(&cargo_toml_content)?;
    Ok(manifest
        .get("profile")
        .and_then(|p| p.as_table())
        .and_then(|p| p.get("samply"))
        .is_some())
}

/// Ensures that the samply profile exists in the given Cargo.toml file.
pub fn ensure_samply_profile(cargo_toml: &Path) -> error::Result<()> {
    if !has_samply_profile_in_manifest(cargo_toml)? {
        let mut f = OpenOptions::new()
            .append(true)
            .open(cargo_toml)
            .path_ctx(cargo_toml)?;
        f.write_all(SAMPLY_PROFILE.as_bytes())
            .path_ctx(cargo_toml)?;
        info!("'samply' profile was added to '{}'", cargo_toml.display());
    }
    Ok(())
}

/// Checks if the samply profile exists in the given Cargo.toml file.
pub fn has_samply_profile(cargo_toml: &Path) -> error::Result<bool> {
    has_samply_profile_in_manifest(cargo_toml)
}

/// Helper to find the package that contains the current working directory.
pub fn find_current_package(
    metadata: &cargo_metadata::Metadata,
) -> Option<&cargo_metadata::Package> {
    let current_dir = std::env::current_dir().ok()?;
    let current_dir = current_dir.canonicalize().ok()?;
    let mut best_match = None;
    let mut best_length = 0;

    for package in &metadata.packages {
        if let Some(package_dir) = package.manifest_path.parent() {
            let package_dir_std = package_dir.as_std_path();
            let package_dir_canon = match package_dir_std.canonicalize() {
                Ok(p) => p,
                Err(_) => continue,
            };

            if current_dir.starts_with(&package_dir_canon) {
                let len = package_dir_canon.as_os_str().len();
                if len > best_length {
                    best_match = Some(package);
                    best_length = len;
                }
            }
        }
    }

    best_match
}

/// Gets workspace metadata including all available binaries and examples.
///
/// # Arguments
///
/// * `cargo_toml` - Path to the Cargo.toml file
/// * `selected_package` - Optional package name to filter targets
///
/// # Errors
///
/// Returns an error if the Cargo.toml cannot be read or parsed,
/// or if the specified package is not found.
pub fn get_workspace_metadata_from(
    cargo_toml: &Path,
    selected_package: Option<&str>,
) -> error::Result<WorkspaceMetadata> {
    let work_dir = cargo_toml.parent().unwrap_or_else(|| Path::new("."));

    let metadata = MetadataCommand::new()
        .current_dir(work_dir)
        .no_deps()
        .exec()?;

    let mut binaries_set = HashSet::new();
    let mut examples_set = HashSet::new();
    let mut benches_set = HashSet::new();
    let mut tests_set = HashSet::new();

    // Determine which packages are relevant
    let relevant_packages: Vec<&cargo_metadata::Package> = if let Some(pkg_name) = selected_package
    {
        let pkg = metadata
            .packages
            .iter()
            .find(|p| p.name == pkg_name)
            .ok_or_else(|| error::Error::PackageNotFound {
                name: pkg_name.to_string(),
            })?;
        vec![pkg]
    } else if let Some(pkg) = find_current_package(&metadata) {
        vec![pkg]
    } else {
        // Fallback to all workspace members if not in a specific package
        let mut pkgs = Vec::new();
        for id in &metadata.workspace_members {
            if let Some(pkg) = metadata.packages.iter().find(|p| p.id == *id) {
                pkgs.push(pkg);
            }
        }
        pkgs
    };

    for package in relevant_packages {
        for target in &package.targets {
            if target.is_bin() {
                binaries_set.insert(target.name.clone());
            } else if target.is_example() {
                examples_set.insert(target.name.clone());
            } else if target.kind.contains(&cargo_metadata::TargetKind::Bench) {
                benches_set.insert(target.name.clone());
            } else if target.kind.contains(&cargo_metadata::TargetKind::Test) {
                tests_set.insert(target.name.clone());
            }
        }
    }

    let mut binaries: Vec<String> = binaries_set.into_iter().collect();
    let mut examples: Vec<String> = examples_set.into_iter().collect();
    let mut benches: Vec<String> = benches_set.into_iter().collect();
    let mut tests: Vec<String> = tests_set.into_iter().collect();

    binaries.sort();
    examples.sort();
    benches.sort();
    tests.sort();

    Ok(WorkspaceMetadata {
        binaries,
        examples,
        benches,
        tests,
        workspace_root: metadata.workspace_root.into(),
    })
}

/// Retrieves all available targets from the workspace.
///
/// # Arguments
///
/// * `cargo_toml` - Path to the Cargo.toml file
/// * `selected_package` - Optional package name to filter targets
///
/// # Errors
///
/// Returns an error if the Cargo.toml cannot be read or parsed,
/// or if the specified package is not found.
pub fn get_all_targets(
    cargo_toml: &Path,
    selected_package: Option<&str>,
) -> error::Result<WorkspaceMetadata> {
    get_workspace_metadata_from(cargo_toml, selected_package)
}

/// Resolves a benchmark target name, validating it exists.
///
/// # Arguments
///
/// * `cargo_toml` - Path to the Cargo.toml file
/// * `requested` - The requested benchmark name
/// * `selected_package` - Optional package name filter
///
/// # Returns
///
/// The validated benchmark name, or the original if not found
/// (allowing cargo to produce the error).
///
/// # Errors
///
/// Returns an error if the Cargo.toml cannot be read or parsed,
/// or if the specified package is not found.
pub fn resolve_bench_target_name(
    cargo_toml: &Path,
    requested: &str,
    selected_package: Option<&str>,
) -> error::Result<String> {
    let targets = get_all_targets(cargo_toml, selected_package)?;
    if let Some(found) = targets
        .benches
        .iter()
        .find(|&candidate| candidate == requested)
    {
        return Ok(found.clone());
    }
    Ok(requested.to_string())
}

/// Attempts to determine which binary to run.
///
/// Uses the following priority:
/// 1. `default-run` from Cargo.toml manifest
/// 2. The only binary (if exactly one exists)
/// 3. Returns an error with suggestions if ambiguous
///
/// # Errors
///
/// Returns `NoBinaryFound` if no binaries exist, or
/// `BinaryToRunNotDetermined` if multiple binaries exist
/// without a default.
pub fn guess_bin(cargo_toml: &Path, all_targets: &WorkspaceMetadata) -> error::Result<String> {
    if let Ok(manifest) = cargo_toml::Manifest::from_path(cargo_toml) {
        let default_run = manifest.package.and_then(|p| p.default_run);
        if let Some(bin) = default_run {
            return Ok(bin);
        }
    }

    if all_targets.binaries.is_empty() {
        return Err(error::Error::NoBinaryFound);
    }

    if all_targets.binaries.len() == 1 {
        return Ok(all_targets.binaries[0].clone());
    }

    create_suggestions_error(all_targets.binaries.clone(), all_targets.examples.clone())
}

/// Helper function to add suggestions for a list of targets.
///
/// # Arguments
///
/// * `suggestions` - Mutable vector to append suggestions to
/// * `targets` - List of target names
/// * `target_type` - Type of target (e.g., "binaries", "examples")
/// * `flag` - Command-line flag to use (e.g., "--bin", "--example")
fn add_target_suggestions(
    suggestions: &mut Vec<String>,
    targets: &[String],
    target_type: &str,
    flag: &str,
) {
    if !targets.is_empty() {
        suggestions.push(format!("\n\nAvailable {}:", target_type));
        for target in targets {
            suggestions.push(format!("  {}: cargo samply {} {}", target, flag, target));
        }
    }
}

fn create_suggestions_error(binaries: Vec<String>, examples: Vec<String>) -> error::Result<String> {
    let mut suggestions = Vec::new();

    add_target_suggestions(&mut suggestions, &binaries, "binaries", "--bin");
    add_target_suggestions(&mut suggestions, &examples, "examples", "--example");

    let suggestions_text = suggestions.join("\n");
    Err(error::Error::BinaryToRunNotDetermined {
        suggestions: suggestions_text,
    })
}

/// Extension trait for `std::process::Command` with logging support.
///
/// Provides convenience methods for running commands with automatic
/// debug logging of the command and arguments.
pub trait CommandExt {
    /// Spawns the command, waits for completion, and returns the exit status.
    ///
    /// Logs the command and arguments at debug level before execution.
    ///
    /// # Errors
    ///
    /// Returns an error if spawning the command fails or if waiting for it fails.
    fn call(&mut self) -> error::Result<ExitStatus>;

    /// Logs the command and arguments at debug level.
    ///
    /// Returns `&mut Command` for method chaining.
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

/// Platform-specific configuration for library path environment variables.
///
/// Different operating systems use different environment variables
/// and path separators for dynamic library loading.
#[derive(Debug, Clone, Copy)]
pub struct Platform {
    /// The environment variable name (e.g., "LD_LIBRARY_PATH", "DYLD_LIBRARY_PATH", "PATH")
    pub env_var_name: &'static str,
    /// The path separator (e.g., ":" on Unix, ";" on Windows)
    pub separator: &'static str,
}

impl Platform {
    pub fn current() -> Self {
        if cfg!(target_os = "macos") {
            Self {
                env_var_name: "DYLD_LIBRARY_PATH",
                separator: ":",
            }
        } else if cfg!(target_os = "windows") {
            Self {
                env_var_name: "PATH",
                separator: ";",
            }
        } else {
            Self {
                env_var_name: "LD_LIBRARY_PATH",
                separator: ":",
            }
        }
    }
}

fn get_rust_sysroot() -> error::Result<PathBuf> {
    let output = Command::new("rustc")
        .arg("--print")
        .arg("sysroot")
        .output()?;
    if !output.status.success() {
        return Err(error::Error::RustSysrootFailed {
            message: String::from_utf8_lossy(&output.stderr).to_string(),
        });
    }
    Ok(PathBuf::from(from_utf8(&output.stdout)?.trim()))
}

fn get_rustc_host_target() -> error::Result<String> {
    let output = Command::new("rustc").arg("-vV").output()?;
    if !output.status.success() {
        return Err(error::Error::RustHostTargetFailed {
            message: String::from_utf8_lossy(&output.stderr).to_string(),
        });
    }
    let output_str = from_utf8(&output.stdout)?;
    for line in output_str.lines() {
        if let Some(host) = line.strip_prefix("host: ") {
            return Ok(host.trim().to_string());
        }
    }
    Err(error::Error::RustHostTargetFailed {
        message: "Could not find 'host:' line in rustc output".to_string(),
    })
}

/// Configures the library path environment for running a binary.
///
/// Adds the Rust sysroot library paths and the binary's deps directory
/// to the appropriate platform-specific environment variable.
///
/// # Arguments
///
/// * `cmd` - The command to configure
/// * `bin_path` - Path to the binary being run
/// * `profile` - The build profile used
///
/// # Errors
///
/// Returns an error if the Rust sysroot cannot be determined.
pub fn configure_library_path_for_binary(
    cmd: &mut Command,
    bin_path: &Path,
    profile: &str,
) -> error::Result<()> {
    if let Some((key, val)) = calculate_library_path(bin_path, profile)? {
        debug!("Setting {} to: {}", key, val);
        cmd.env(key, val);
    }
    Ok(())
}

/// Calculates the library path value for a given binary.
///
/// Returns the environment variable name and value to set,
/// or `None` if sysroot injection is disabled via
/// `CARGO_SAMPLY_NO_SYSROOT_INJECTION`.
///
/// # Arguments
///
/// * `bin_path` - Path to the binary being run
/// * `profile` - The build profile used
///
/// # Errors
///
/// Returns an error if the Rust sysroot cannot be determined.
pub fn calculate_library_path(
    bin_path: &Path,
    profile: &str,
) -> error::Result<Option<(String, String)>> {
    let mut extra_paths = Vec::new();
    if let Some(bin_dir) = bin_path.parent() {
        if bin_dir.file_name().is_some_and(|name| name == "deps") {
            extra_paths.push(bin_dir.to_path_buf());
        } else {
            extra_paths.push(bin_dir.join("deps"));
        }
    }
    let target_triple = infer_target_triple(bin_path, profile);
    calculate_library_path_impl_pure(&extra_paths, &target_triple, Platform::current())
}

fn infer_target_triple(bin_path: &Path, profile: &str) -> String {
    let components: Vec<_> = bin_path.components().collect();
    if let Some(target_idx) = components.iter().position(|c| c.as_os_str() == "target") {
        if let Some(triple) = components.get(target_idx + 1) {
            if let Some(prof) = components.get(target_idx + 2) {
                if prof.as_os_str() == profile {
                    return triple.as_os_str().to_string_lossy().into_owned();
                }
            }
        }
    }
    get_rustc_host_target().unwrap_or_else(|_| "unknown".to_string())
}

fn calculate_library_path_impl_pure(
    extra_paths: &[PathBuf],
    target_triple: &str,
    platform: Platform,
) -> error::Result<Option<(String, String)>> {
    if std::env::var("CARGO_SAMPLY_NO_SYSROOT_INJECTION").is_ok() {
        return Ok(None);
    }
    let sysroot = get_rust_sysroot()?;
    let env_var_name = platform.env_var_name;
    let separator = platform.separator;
    let current_val = std::env::var_os(env_var_name)
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let lib_path = sysroot.join("lib");
    let target_lib_path = sysroot
        .join("lib")
        .join("rustlib")
        .join(target_triple)
        .join("lib");
    let mut parts: Vec<String> = Vec::new();
    let mut seen = std::collections::HashSet::new();
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
    Ok(Some((env_var_name.to_string(), parts.join(separator))))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_ensure_samply_profile_already_exists() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_toml_path = temp_dir.path().join("Cargo.toml");
        let initial_content = r#"[package]
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
        assert_eq!(original_content, new_content);
    }

    #[test]
    fn test_guess_bin_single_bin() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_toml_path = temp_dir.path().join("Cargo.toml");
        let content = r#"[package]
name = "test"
version = "0.1.0"
[[bin]]
name = "single"
path = "src/main.rs"
"#;
        fs::write(&cargo_toml_path, content).unwrap();
        // Since we are not in a workspace here, it might fallback to all targets
        // but we need src/main.rs to exist for metadata if we don't specify it
        let src_dir = temp_dir.path().join("src");
        fs::create_dir(&src_dir).unwrap();
        fs::write(src_dir.join("main.rs"), "").unwrap();

        let metadata = get_all_targets(&cargo_toml_path, None).unwrap();
        let bin = guess_bin(&cargo_toml_path, &metadata).unwrap();
        assert_eq!(bin, "single");
    }
}
