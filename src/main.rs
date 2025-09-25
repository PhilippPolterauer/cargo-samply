//! Main entry point for the cargo-samply binary.
//!
//! This module handles command-line argument parsing and coordinates
//! the build and profiling process.

#[macro_use]
extern crate log;

mod cli;
mod error;
mod util;

use std::process::Command;
use std::vec;

use clap::Parser;

use crate::util::{ensure_samply_profile, guess_bin, locate_project, CommandExt};

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
        return path;
    }

    path
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

    let crate::cli::CargoCli::Samply(cli) = cli;
    let log_level = if cli.quiet {
        log::Level::Error
    } else if cli.verbose {
        log::Level::Debug
    } else {
        log::Level::Warn
    };
    ocli::init(log_level)?;

    if cli.bin.is_some() && cli.example.is_some() {
        return Err(error::Error::BinAndExampleMutuallyExclusive);
    }

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

    let (bin_opt, bin_name) = if let Some(bin) = cli.bin {
        ("--bin", bin)
    } else if let Some(example) = cli.example {
        ("--example", example)
    } else {
        ("--bin", guess_bin(&cargo_toml)?)
    };

    let features_str = if !cli.features.is_empty() {
        Some(cli.features.join(","))
    } else {
        None
    };

    let mut args = vec!["build", "--profile", &cli.profile, &bin_opt, &bin_name];
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
    let bin_path = get_bin_path(root, &cli.profile, bin_opt, &bin_name);

    if !bin_path.exists() {
        return Err(error::Error::BinaryNotFound { path: bin_path });
    }

    if !cli.no_samply {
        let samply_available = std::process::Command::new("samply")
            .arg("--help")
            .status()
            .is_ok();
        if !samply_available {
            return Err(error::Error::SamplyNotFound);
        }
        Command::new("samply")
            .arg("record")
            .arg(bin_path)
            .args(cli.args)
            .call()?;
    } else {
        Command::new(bin_path).args(cli.args).call()?;
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
}
