//! Main entry point for the cargo-samply binary.
//!
//! This module handles command-line argument parsing and coordinates
//! the build and profiling process.

#[macro_use]
extern crate log;

mod cli;
mod error;
mod util;

use std::fs;
use std::mem;
use std::process::Command;
use std::time::SystemTime;
use std::vec;

use clap::Parser;

use crate::util::{
    bench_uses_harness, ensure_samply_profile, guess_bin, locate_project,
    resolve_bench_target_name, CommandExt,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TargetKind {
    Bin,
    Example,
    Bench,
}

impl TargetKind {
    fn cargo_flag(self) -> &'static str {
        match self {
            TargetKind::Bin => "--bin",
            TargetKind::Example => "--example",
            TargetKind::Bench => "--bench",
        }
    }
}

#[derive(Debug, Clone)]
struct Target {
    kind: TargetKind,
    name: String,
}

impl Target {
    fn new(kind: TargetKind, name: String) -> Self {
        Self { kind, name }
    }
}

/// Constructs the path to the built binary based on profile and binary type.
///
/// # Arguments
///
/// * `root` - Project root directory
/// * `profile` - Build profile (e.g., "debug", "release", "samply")
/// * `bin_opt` - Binary option ("--bin" or "--example")
/// * `bin_name` - Name of the binary or example
///
/// # Returns
///
/// Path to the built binary in the target directory
fn get_bin_path(
    root: &std::path::Path,
    profile: &str,
    bin_opt: &str,
    bin_name: &str,
) -> std::path::PathBuf {
    let path = if bin_opt == "--bin" {
        root.join("target").join(profile).join(bin_name)
    } else {
        root.join("target")
            .join(profile)
            .join("examples")
            .join(bin_name)
    };

    // On Windows, built executables have the `.exe` extension. Append it
    // when running on that platform to make existence checks and command
    // invocation work correctly.
    #[cfg(windows)]
    {
        let mut path = path;
        {
            if path.extension().is_none() {
                path.set_extension("exe");
            }
        }
        path
    }
    #[cfg(not(windows))]
    {
        path
    }
}

fn resolve_target_path(
    root: &std::path::Path,
    profile: &str,
    target: &Target,
) -> error::Result<std::path::PathBuf> {
    match target.kind {
        TargetKind::Bin => Ok(get_bin_path(
            root,
            profile,
            TargetKind::Bin.cargo_flag(),
            &target.name,
        )),
        TargetKind::Example => Ok(get_bin_path(
            root,
            profile,
            TargetKind::Example.cargo_flag(),
            &target.name,
        )),
        TargetKind::Bench => get_bench_path(root, profile, &target.name),
    }
}

fn determine_target(
    cli: &crate::cli::Config,
    cargo_toml: &std::path::Path,
) -> error::Result<Target> {
    let specified =
        cli.bin.is_some() as u8 + cli.example.is_some() as u8 + cli.bench.is_some() as u8;
    if specified > 1 {
        return Err(error::Error::BinAndExampleMutuallyExclusive);
    }

    if let Some(bin) = &cli.bin {
        return Ok(Target::new(TargetKind::Bin, bin.clone()));
    }
    if let Some(example) = &cli.example {
        return Ok(Target::new(TargetKind::Example, example.clone()));
    }
    if let Some(bench) = &cli.bench {
        let resolved = resolve_bench_target_name(cargo_toml, bench)?;
        return Ok(Target::new(TargetKind::Bench, resolved));
    }

    Ok(Target::new(TargetKind::Bin, guess_bin(cargo_toml)?))
}

fn get_bench_path(
    root: &std::path::Path,
    profile: &str,
    bench_name: &str,
) -> error::Result<std::path::PathBuf> {
    let deps_dir = root.join("target").join(profile).join("deps");
    if !deps_dir.exists() {
        return Err(error::Error::BinaryNotFound {
            path: deps_dir.join(bench_name),
        });
    }

    let mut prefixes = vec![format!("{bench_name}-")];
    let sanitized = bench_name.replace('-', "_");
    if sanitized != bench_name {
        prefixes.push(format!("{sanitized}-"));
    }
    let mut newest: Option<(SystemTime, std::path::PathBuf)> = None;

    for entry in fs::read_dir(&deps_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let path = entry.path();
        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if !prefixes.iter().any(|prefix| file_name.starts_with(prefix)) {
            continue;
        }
        if !is_executable_artifact(&path) {
            continue;
        }
        let modified = entry
            .metadata()?
            .modified()
            .unwrap_or(SystemTime::UNIX_EPOCH);

        match &mut newest {
            Some((ts, best_path)) if modified > *ts => {
                *ts = modified;
                *best_path = path;
            }
            None => newest = Some((modified, path)),
            _ => {}
        }
    }

    newest
        .map(|(_, path)| path)
        .ok_or_else(|| error::Error::BinaryNotFound {
            path: deps_dir.join(format!("{bench_name}-*")),
        })
}

fn is_executable_artifact(path: &std::path::Path) -> bool {
    if cfg!(windows) {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("exe"))
            .unwrap_or(false)
    } else {
        path.extension().is_none()
    }
}

fn prepare_runtime_args(
    target: &Target,
    bench_requires_flag: bool,
    trailing_args: Vec<String>,
) -> Vec<String> {
    let mut args = Vec::new();
    if bench_requires_flag {
        args.push("--bench".to_string());
        args.push(target.name.clone());
    }
    args.extend(trailing_args);
    args
}

/// Entry point for the cargo-samply application.
///
/// Initializes error handling and calls the main run function.
fn main() {
    if let Err(err) = run() {
        error!("{}", err);
        std::process::exit(1);
    }
}

/// Main application logic for cargo-samply.
///
/// This function orchestrates the entire process:
/// 1. Parse command-line arguments
/// 2. Set up logging
/// 3. Validate arguments
/// 4. Locate the cargo project
/// 5. Ensure the samply profile exists
/// 6. Determine which binary to run
/// 7. Build the project
/// 8. Run samply or the binary directly
///
/// # Returns
///
/// - `Ok(())` - Operation completed successfully
/// - `Err(Error)` - Various errors can occur during the process
fn run() -> error::Result<()> {
    // Handle both direct execution and cargo subcommand
    let args: Vec<String> = std::env::args().collect();
    let cli = if args.len() > 1 && args[1] == "samply" {
        // Called via cargo: cargo samply [args...]
        crate::cli::CargoCli::parse()
    } else {
        // Called directly: cargo-samply [args...]
        // Parse as if "samply" was the first argument
        let mut modified_args = vec!["cargo".to_string(), "samply".to_string()];
        modified_args.extend(args.into_iter().skip(1));
        crate::cli::CargoCli::try_parse_from(modified_args)
            .map_err(|e| {
                eprintln!("{}", e);
                std::process::exit(1);
            })
            .unwrap()
    };

    let crate::cli::CargoCli::Samply(mut cli) = cli;
    let log_level = if cli.quiet {
        log::Level::Error
    } else if cli.verbose {
        log::Level::Debug
    } else {
        log::Level::Warn
    };
    ocli::init(log_level)?;

    // check if cargo.toml exists
    // check project path using locate-project
    let cargo_toml = locate_project()?;
    debug!("cargo.toml: {:?}", cargo_toml);

    // check if profile exists
    // if not add profile
    // if yes print warning
    if cli.profile == "samply" {
        ensure_samply_profile(&cargo_toml)?;
    }

    let target = determine_target(&cli, &cargo_toml)?;
    let bench_requires_flag =
        matches!(target.kind, TargetKind::Bench) && bench_uses_harness(&cargo_toml, &target.name)?;

    let features_str = if !cli.features.is_empty() {
        Some(cli.features.join(","))
    } else {
        None
    };

    let mut args = vec![
        "build",
        "--profile",
        &cli.profile,
        target.kind.cargo_flag(),
        &target.name,
    ];
    if let Some(ref features) = features_str {
        args.push("--features");
        args.push(features);
    }
    if cli.no_default_features {
        args.push("--no-default-features");
    }
    let exit_code = Command::new("cargo").args(args).call()?;
    if !exit_code.success() {
        return Err(error::Error::CargoBuildFailed);
    }

    // run samply on the binary
    // if it fails print error
    let root = cargo_toml.parent().unwrap();
    let bin_path = resolve_target_path(root, &cli.profile, &target)?;

    if !bin_path.exists() {
        return Err(error::Error::BinaryNotFound { path: bin_path });
    }

    let runtime_args = prepare_runtime_args(&target, bench_requires_flag, mem::take(&mut cli.args));

    if !cli.no_samply {
        let samply_available = std::process::Command::new("samply")
            .arg("--help")
            .status()
            .is_ok();
        if !samply_available {
            return Err(error::Error::SamplyNotFound);
        }
        let mut samply_cmd = Command::new("samply");
        samply_cmd.arg("record").arg(&bin_path);
        if !runtime_args.is_empty() {
            samply_cmd.arg("--");
            samply_cmd.args(&runtime_args);
        }
        samply_cmd.call()?;
    } else {
        Command::new(&bin_path).args(&runtime_args).call()?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multiple_features_handling() {
        // Test multiple features passed as separate flags
        let cli = crate::cli::Config {
            args: vec![],
            profile: "samply".to_string(),
            bin: Some("test".to_string()),
            example: None,
            bench: None,
            features: vec!["feature1".to_string(), "feature2".to_string()],
            no_default_features: false,
            verbose: false,
            quiet: false,
            no_samply: false,
        };

        let features_str = if !cli.features.is_empty() {
            Some(cli.features.join(","))
        } else {
            None
        };

        assert_eq!(features_str, Some("feature1,feature2".to_string()));
    }

    #[test]
    fn test_single_feature_handling() {
        // Test single feature
        let cli = crate::cli::Config {
            args: vec![],
            profile: "samply".to_string(),
            bin: Some("test".to_string()),
            example: None,
            bench: None,
            features: vec!["feature1".to_string()],
            no_default_features: false,
            verbose: false,
            quiet: false,
            no_samply: false,
        };

        let features_str = if !cli.features.is_empty() {
            Some(cli.features.join(","))
        } else {
            None
        };

        assert_eq!(features_str, Some("feature1".to_string()));
    }

    #[test]
    fn test_no_features_handling() {
        // Test no features
        let cli = crate::cli::Config {
            args: vec![],
            profile: "samply".to_string(),
            bin: Some("test".to_string()),
            example: None,
            bench: None,
            features: vec![],
            no_default_features: false,
            verbose: false,
            quiet: false,
            no_samply: false,
        };

        let features_str = if !cli.features.is_empty() {
            Some(cli.features.join(","))
        } else {
            None
        };

        assert_eq!(features_str, None);
    }

    #[test]
    fn test_get_bin_path_bin() {
        let root = std::path::Path::new("/project");
        let path = get_bin_path(root, "release", "--bin", "mybin");
        let expected = if cfg!(windows) {
            std::path::Path::new("/project/target/release/mybin.exe")
        } else {
            std::path::Path::new("/project/target/release/mybin")
        };
        assert_eq!(path, expected);
    }

    #[test]
    fn test_get_bin_path_example() {
        let root = std::path::Path::new("/project");
        let path = get_bin_path(root, "debug", "--example", "myexample");
        let expected = if cfg!(windows) {
            std::path::Path::new("/project/target/debug/examples/myexample.exe")
        } else {
            std::path::Path::new("/project/target/debug/examples/myexample")
        };
        assert_eq!(path, expected);
    }

    #[test]
    fn test_prepare_runtime_args_injects_bench_flag() {
        let target = Target::new(TargetKind::Bench, "speed".to_string());
        let args = prepare_runtime_args(&target, true, vec!["--foo".to_string()]);
        assert_eq!(
            args,
            vec![
                "--bench".to_string(),
                "speed".to_string(),
                "--foo".to_string()
            ]
        );
    }

    #[test]
    fn test_prepare_runtime_args_passthrough_for_non_bench() {
        let target = Target::new(TargetKind::Bin, "demo".to_string());
        let args = prepare_runtime_args(&target, false, vec!["--foo".to_string()]);
        assert_eq!(args, vec!["--foo".to_string()]);
    }
}
