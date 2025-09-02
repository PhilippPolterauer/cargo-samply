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
        let mut f = OpenOptions::new()
            .append(true)
            .open(cargo_toml)
            .path_ctx(cargo_toml)?;
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
        Ok(manifest.bin.first().unwrap().name.clone().unwrap())
    } else if manifest.bin.is_empty() {
        Err(error::Error::NoBinaryFound)
    } else {
        Err(error::Error::BinaryToRunNotDetermined)
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

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
        if let Err(error::Error::BinaryToRunNotDetermined) = result {
            // Correct
        } else {
            panic!("Expected BinaryToRunNotDetermined");
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
            panic!("Expected NoBinaryFound");
        }
    }
}
