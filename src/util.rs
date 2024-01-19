use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process::{Command, ExitStatus},
    str::{from_utf8, FromStr},
};

use crate::error::{self, IOResultExt};

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

const SAMPLY_PROFILE: &str = "
[profile.samply]
inherits = \"release\"
debug = true
";

pub fn ensure_samply_profile(cargo_toml: &Path) -> error::Result<()> {
    let cargo_toml_content: String = fs::read_to_string(cargo_toml).path_ctx(cargo_toml)?;
    let manifest = toml::Table::from_str(&cargo_toml_content)?;
    let profile_samply = manifest
        .get("profile")
        .and_then(|p| p.as_table())
        .and_then(|p| p.get("samply"));

    if profile_samply.is_none() {
        let mut f = OpenOptions::new().append(true).open(cargo_toml).unwrap();
        f.write(SAMPLY_PROFILE.as_bytes()).path_ctx(cargo_toml)?;
        info!("'samply' profile was added to 'Cargo.toml'");
    }
    Ok(())
}

pub fn guess_bin(cargo_toml: &Path) -> error::Result<String> {
    let manifest = cargo_toml::Manifest::from_path(cargo_toml)?;
    let default_run = manifest.package.and_then(|p| p.default_run);
    if let Some(bin) = default_run {
        Ok(bin)
    } else if manifest.bin.len() == 1 {
        return Ok(manifest.bin.first().unwrap().name.clone().unwrap());
    } else if manifest.bin.is_empty() {
        return Err(error::Error::NoBinaryFound);
    } else {
        return Err(error::Error::BinaryToRunNotDetermined);
    }
}

/// Extension trait for `Command` that add a `call` method which logs the command in debug mode.
pub trait CommandExt {
    fn call(&mut self) -> error::Result<ExitStatus>;
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
