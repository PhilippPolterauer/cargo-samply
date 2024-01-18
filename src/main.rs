#[macro_use]
extern crate log;

mod cli;
mod error;
mod util;

use std::process::Command;
use std::vec;

use clap::Parser;

use crate::util::{ensure_samply_profile, guess_bin, locate_project, CommandExt};

fn main() {
    if let Err(err) = run() {
        error!("{}", err);
        std::process::exit(1);
    }
}

fn run() -> error::Result<()> {
    let cli = cli::Config::parse();
    ocli::init(if cli.verbose {
        log::Level::Debug
    } else {
        log::Level::Info
    })?;
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

    let mut args = vec!["build", "--profile", &cli.profile, &bin_opt, &bin_name];
    if let Some(features) = cli.features.as_ref() {
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
    let bin_path = root.join("target").join(&cli.profile).join(&bin_name);
    Command::new("samply").arg("record").arg(bin_path).call()?;

    Ok(())
}
