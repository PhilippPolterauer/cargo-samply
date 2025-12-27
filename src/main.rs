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
use std::io::{self, BufReader};
use std::mem;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::SystemTime;
use std::vec;

use cargo_metadata::{Message, TargetKind as CargoTargetKind};
use clap::Parser;

use crate::util::{
    calculate_library_path, configure_library_path_for_binary, ensure_samply_profile,
    get_all_targets, guess_bin, has_samply_profile, locate_project, resolve_bench_target_name,
    CommandExt, WorkspaceMetadata,
};

const SAMPLY_OVERRIDE_ENV: &str = "CARGO_SAMPLY_SAMPLY_PATH";

#[derive(Debug)]
struct BuildPlan {
    cargo_args: Vec<String>,
}

#[derive(Debug)]
struct RunPlan {
    bin_path: PathBuf,
    args: Vec<String>,
    env_vars: Vec<(String, String)>,
    is_samply: bool,
    samply_program: String,
    samply_args: Vec<String>,
    re_resolve_context: Option<(PathBuf, String, Target)>,
}

#[derive(Debug)]
struct ExecutionPlan {
    build: Option<BuildPlan>,
    run: RunPlan,
    warnings: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TargetKind {
    Bin,
    Example,
    Bench,
    Test,
}

impl TargetKind {
    fn cargo_flag(self) -> &'static str {
        match self {
            TargetKind::Bin => "--bin",
            TargetKind::Example => "--example",
            TargetKind::Bench => "--bench",
            TargetKind::Test => "--test",
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
fn get_bin_path(
    root: &std::path::Path,
    profile: &str,
    bin_opt: &str,
    bin_name: &str,
    is_windows: bool,
) -> std::path::PathBuf {
    let path = if bin_opt == "--bin" {
        root.join("target").join(profile).join(bin_name)
    } else {
        root.join("target")
            .join(profile)
            .join("examples")
            .join(bin_name)
    };

    if is_windows {
        let mut path = path;
        if path.extension().is_none() {
            path.set_extension("exe");
        }
        path
    } else {
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
            cfg!(windows),
        )),
        TargetKind::Example => Ok(get_bin_path(
            root,
            profile,
            TargetKind::Example.cargo_flag(),
            &target.name,
            cfg!(windows),
        )),
        TargetKind::Bench | TargetKind::Test => get_bench_path(root, profile, &target.name),
    }
}

fn determine_target(
    cli: &crate::cli::Config,
    cargo_toml: &std::path::Path,
) -> error::Result<(Target, WorkspaceMetadata)> {
    let specified = cli.bin.is_some() as u8
        + cli.example.is_some() as u8
        + cli.bench.is_some() as u8
        + cli.test.is_some() as u8;
    if specified > 1 {
        return Err(error::Error::MultipleTargetsFlagsSpecified);
    }

    let metadata = get_all_targets(cargo_toml, cli.package.as_deref())?;

    if let Some(bin) = &cli.bin {
        return Ok((Target::new(TargetKind::Bin, bin.clone()), metadata));
    }
    if let Some(example) = &cli.example {
        return Ok((Target::new(TargetKind::Example, example.clone()), metadata));
    }
    if let Some(bench) = &cli.bench {
        let resolved = resolve_bench_target_name(cargo_toml, bench, cli.package.as_deref())?;
        return Ok((Target::new(TargetKind::Bench, resolved), metadata));
    }
    if let Some(test) = &cli.test {
        return Ok((Target::new(TargetKind::Test, test.clone()), metadata));
    }

    let bin = guess_bin(cargo_toml, &metadata)?;
    Ok((Target::new(TargetKind::Bin, bin), metadata))
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

fn prepare_runtime_args(bench_flag: Option<&str>, trailing_args: Vec<String>) -> Vec<String> {
    let mut args = Vec::new();
    if let Some(flag) = bench_flag {
        args.push(flag.to_string());
    }
    args.extend(trailing_args);
    args
}

fn configure_samply_command(
    cmd: &mut Command,
    bin_path: &std::path::Path,
    runtime_args: &[String],
    samply_args: &[String],
    profile: &str,
) -> error::Result<()> {
    cmd.arg("record");
    cmd.args(samply_args);
    cmd.arg("--").arg(bin_path);
    if !runtime_args.is_empty() {
        cmd.args(runtime_args);
    }
    configure_library_path_for_binary(cmd, bin_path, profile)?;
    Ok(())
}

fn features_to_string(features: &[String]) -> Option<String> {
    if !features.is_empty() {
        Some(features.join(","))
    } else {
        None
    }
}

fn main() {
    if let Err(err) = run() {
        error!("{}", err);
        std::process::exit(1);
    }
}

fn run() -> error::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let cli = if args.len() > 1 && args[1] == "samply" {
        crate::cli::CargoCli::parse()
    } else {
        let mut modified_args = vec!["cargo".to_string(), "samply".to_string()];
        modified_args.extend(args.into_iter().skip(1));
        crate::cli::CargoCli::try_parse_from(modified_args).unwrap_or_else(|e| e.exit())
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

    let local_cargo_toml = locate_project()?;
    debug!("local cargo.toml: {:?}", local_cargo_toml);

    if cli.list_targets {
        let targets = get_all_targets(&local_cargo_toml, cli.package.as_deref())?;
        if !targets.binaries.is_empty() {
            println!("Binaries:");
            for bin in targets.binaries {
                println!("  {}", bin);
            }
        }
        if !targets.examples.is_empty() {
            println!("Examples:");
            for example in targets.examples {
                println!("  {}", example);
            }
        }
        if !targets.benches.is_empty() {
            println!("Benches:");
            for bench in targets.benches {
                println!("  {}", bench);
            }
        }
        if !targets.tests.is_empty() {
            println!("Tests:");
            for test in targets.tests {
                println!("  {}", test);
            }
        }
        return Ok(());
    }

    let plan = generate_plan(&mut cli, &local_cargo_toml)?;

    if cli.dry_run {
        print_plan(&plan);
    } else {
        execute_plan(plan, &cli.profile, &local_cargo_toml)?;
    }

    Ok(())
}

fn generate_plan(
    cli: &mut crate::cli::Config,
    cargo_toml: &std::path::Path,
) -> error::Result<ExecutionPlan> {
    let mut warnings = Vec::new();

    let samply_program =
        std::env::var(SAMPLY_OVERRIDE_ENV).unwrap_or_else(|_| "samply".to_string());

    if !cli.no_samply && !cli.dry_run && which::which(&samply_program).is_err() {
        return Err(error::Error::SamplyNotFound);
    }

    let (target, metadata) = determine_target(cli, cargo_toml)?;
    let workspace_root = &metadata.workspace_root;

    // Profile injection logic
    if cli.profile == "samply" {
        let no_inject_env = std::env::var("CARGO_SAMPLY_NO_PROFILE_INJECT").is_ok();
        let should_inject = !cli.no_profile_inject && !no_inject_env;

        if should_inject {
            if !cli.dry_run {
                // In a workspace, ensure the profile is in the workspace root
                let workspace_cargo_toml = workspace_root.join("Cargo.toml");
                ensure_samply_profile(&workspace_cargo_toml)?;
            }
        } else {
            let workspace_cargo_toml = workspace_root.join("Cargo.toml");
            if !has_samply_profile(&workspace_cargo_toml)? {
                warnings.push("Warning: Profile 'samply' is missing in Cargo.toml and injection is disabled. Profiling might fail or lack symbols.".to_string());
            }
        }
    }

    let features_str = features_to_string(&cli.features);

    let mut cargo_args = vec![
        "build".to_string(),
        "--message-format=json-diagnostic-rendered-ansi".to_string(),
        "--profile".to_string(),
        cli.profile.clone(),
    ];

    if let Some(package) = &cli.package {
        cargo_args.push("--package".to_string());
        cargo_args.push(package.clone());
    }

    cargo_args.push(target.kind.cargo_flag().to_string());
    cargo_args.push(target.name.clone());

    if let Some(ref features) = features_str {
        cargo_args.push("--features".to_string());
        cargo_args.push(features.clone());
    }
    if cli.no_default_features {
        cargo_args.push("--no-default-features".to_string());
    }

    let build_plan = BuildPlan { cargo_args };

    // Run Plan
    let (bin_path, re_resolve_context) =
        match resolve_target_path(workspace_root, &cli.profile, &target) {
            Ok(p) => (p, None),
            Err(error::Error::BinaryNotFound { path }) => (
                path,
                Some((workspace_root.clone(), cli.profile.clone(), target.clone())),
            ),
            Err(e) => return Err(e),
        };

    let bench_flag = if matches!(target.kind, TargetKind::Bench) {
        if cli.bench_flag == "none" {
            None
        } else {
            Some(cli.bench_flag.as_str())
        }
    } else {
        None
    };

    let runtime_args = prepare_runtime_args(bench_flag, mem::take(&mut cli.args));

    let env_vars_opt = calculate_library_path(&bin_path, &cli.profile)?;
    let mut env_vars = Vec::new();
    if let Some((k, v)) = env_vars_opt {
        env_vars.push((k, v));
    }

    let samply_args = cli
        .samply_args
        .as_ref()
        .map(|s| shell_words::split(s))
        .transpose()
        .map_err(|e| {
            error::Error::Io(std::io::Error::other(format!("Invalid samply-args: {}", e)))
        })?
        .unwrap_or_default();

    let run_plan = RunPlan {
        bin_path,
        args: runtime_args,
        env_vars,
        is_samply: !cli.no_samply,
        samply_program,
        samply_args,
        re_resolve_context,
    };

    Ok(ExecutionPlan {
        build: Some(build_plan),
        run: run_plan,
        warnings,
    })
}

fn print_plan(plan: &ExecutionPlan) {
    for w in &plan.warnings {
        eprintln!("{}", w);
    }

    if let Some(build) = &plan.build {
        let quoted_args: Vec<String> = build
            .cargo_args
            .iter()
            .map(|s| shell_words::quote(s).into_owned())
            .collect();
        println!("cargo {}", quoted_args.join(" "));
    }

    let run = &plan.run;
    let mut cmd_parts = Vec::new();

    for (k, v) in &run.env_vars {
        cmd_parts.push(format!("{}={}", k, shell_words::quote(v)));
    }

    if run.is_samply {
        cmd_parts.push(shell_words::quote(&run.samply_program).into_owned());
        cmd_parts.push("record".to_string());
        for arg in &run.samply_args {
            cmd_parts.push(shell_words::quote(arg).into_owned());
        }
        cmd_parts.push("--".to_string());
    }

    cmd_parts.push(shell_words::quote(&run.bin_path.display().to_string()).into_owned());

    for arg in &run.args {
        cmd_parts.push(shell_words::quote(arg).into_owned());
    }

    println!("{}", cmd_parts.join(" "));
}

fn execute_plan(
    plan: ExecutionPlan,
    profile: &str,
    _cargo_toml: &std::path::Path,
) -> error::Result<()> {
    for w in &plan.warnings {
        eprintln!("{}", w);
    }

    let mut bin_path_from_build: Option<PathBuf> = None;
    let target_name = plan
        .run
        .re_resolve_context
        .as_ref()
        .map(|(_, _, t)| t.name.clone());

    if let Some(build) = plan.build {
        let mut cmd = Command::new("cargo");
        cmd.args(&build.cargo_args);
        cmd.stdout(Stdio::piped());

        debug!(
            "running {:?} with args: {:?}",
            cmd.get_program(),
            cmd.get_args().collect::<Vec<_>>()
        );

        let mut child = cmd.spawn()?;
        let stdout = child.stdout.take().unwrap();
        let reader = BufReader::new(stdout);

        for message in Message::parse_stream(reader) {
            match message? {
                Message::CompilerMessage(msg) => {
                    if let Some(rendered) = msg.message.rendered {
                        eprint!("{}", rendered);
                    }
                }
                Message::CompilerArtifact(artifact) => {
                    if let Some(name) = &target_name {
                        if &artifact.target.name == name
                            && artifact.target.kind.iter().any(|k| {
                                k == &CargoTargetKind::Bin
                                    || k == &CargoTargetKind::Example
                                    || k == &CargoTargetKind::Bench
                                    || k == &CargoTargetKind::Test
                            })
                        {
                            if let Some(path) = artifact.executable {
                                bin_path_from_build = Some(path.into());
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        let exit_code = child.wait()?;
        if !exit_code.success() {
            return Err(error::Error::CargoBuildFailed);
        }
    }

    let bin_path = if let Some(path) = bin_path_from_build {
        path
    } else if let Some((root, profile, target)) = plan.run.re_resolve_context {
        resolve_target_path(&root, &profile, &target)?
    } else {
        plan.run.bin_path
    };

    if !bin_path.exists() {
        return Err(error::Error::BinaryNotFound { path: bin_path });
    }

    if plan.run.is_samply {
        let mut samply_cmd = Command::new(&plan.run.samply_program);
        configure_samply_command(
            &mut samply_cmd,
            &bin_path,
            &plan.run.args,
            &plan.run.samply_args,
            profile,
        )?;
        match samply_cmd.call() {
            Ok(_) => {}
            Err(error::Error::Io(io_err)) if io_err.kind() == io::ErrorKind::NotFound => {
                return Err(error::Error::SamplyNotFound);
            }
            Err(err) => return Err(err),
        }
    } else {
        let mut cmd = Command::new(&bin_path);
        cmd.args(&plan.run.args);
        configure_library_path_for_binary(&mut cmd, &bin_path, profile)?;
        cmd.call()?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{ffi::OsString, path::Path};

    fn test_config(features: Vec<String>) -> crate::cli::Config {
        crate::cli::Config {
            args: vec![],
            profile: "samply".to_string(),
            package: None,
            bin: Some("test".to_string()),
            example: None,
            bench: None,
            test: None,
            features,
            no_default_features: false,
            verbose: false,
            quiet: false,
            no_samply: false,
            dry_run: false,
            no_profile_inject: false,
            bench_flag: "--bench".to_string(),
            samply_args: None,
            list_targets: false,
        }
    }

    #[test]
    fn test_multiple_features_handling() {
        let cli = test_config(vec!["feature1".to_string(), "feature2".to_string()]);
        let features_str = features_to_string(&cli.features);
        assert_eq!(features_str, Some("feature1,feature2".to_string()));
    }

    #[test]
    fn test_single_feature_handling() {
        let cli = test_config(vec!["feature1".to_string()]);
        let features_str = features_to_string(&cli.features);
        assert_eq!(features_str, Some("feature1".to_string()));
    }

    #[test]
    fn test_no_features_handling() {
        let cli = test_config(vec![]);
        let features_str = features_to_string(&cli.features);
        assert_eq!(features_str, None);
    }

    #[test]
    fn samply_command_places_binary_before_separator() {
        let mut cmd = Command::new("samply");
        let runtime_args = vec!["--bench".to_string(), "throughput".to_string()];
        configure_samply_command(
            &mut cmd,
            Path::new("target/bin"),
            &runtime_args,
            &[],
            "samply",
        )
        .unwrap();
        let args: Vec<OsString> = cmd.get_args().map(|arg| arg.to_os_string()).collect();

        let expected = vec![
            OsString::from("record"),
            OsString::from("--"),
            OsString::from("target/bin"),
            OsString::from("--bench"),
            OsString::from("throughput"),
        ];

        assert_eq!(args, expected);
    }

    #[test]
    fn samply_command_inserts_separator_even_without_runtime_args() {
        let mut cmd = Command::new("samply");
        configure_samply_command(&mut cmd, Path::new("target/bin"), &[], &[], "samply").unwrap();
        let args: Vec<OsString> = cmd.get_args().map(|arg| arg.to_os_string()).collect();

        let expected = vec![
            OsString::from("record"),
            OsString::from("--"),
            OsString::from("target/bin"),
        ];

        assert_eq!(args, expected);
    }
}
