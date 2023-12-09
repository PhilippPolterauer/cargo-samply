// i want to write a programm that adds a new profile to cargo.toml if it does not exist
// if it adds a profile it should put out a warning
use serde::Deserialize;
use serde_json;
use std::fs;
use std::process::Command;
use std::str::{from_utf8, FromStr};
use toml;

#[derive(Deserialize)]
struct LocateProject {
    root: String,
}

fn samply_profile_default() -> toml::Value {
    let inherits = toml::Value::String("release".to_owned());
    let debug = toml::Value::Boolean(true);
    toml::Value::Table(toml::Table::from_iter(vec![
        ("inherits".to_string(), inherits),
        ("debug".to_string(), debug),
    ]))
}

fn main() {
    // check if cargo.toml exists
    // check projct path using locate-project

    let output = Command::new("cargo")
        .arg("locate-project")
        .output()
        .expect("failed to run 'cargo locate-project'");

    let result: LocateProject = serde_json::from_str(from_utf8(&output.stdout).unwrap()).unwrap();
    let cargo_toml = result.root;

    // check if cargo.toml exists
    println!("cargo.toml: {}", cargo_toml);
    // let file = File::open(root).expect("cargo.toml does not exist");
    // check if profile exists
    // if not add profile
    // if yes print warning

    let binding: String = fs::read_to_string(&cargo_toml)
        .expect(format!("failed reading '{}'", &cargo_toml).as_str());
    let cargo_toml_content = binding.as_str();

    let mut manifest_toml = toml::Table::from_str(cargo_toml_content).unwrap();

    let profile = manifest_toml
        .entry("profile")
        .or_insert(toml::Value::Table(toml::Table::new()));

    profile
        .as_table_mut()
        .expect("profile is not a table")
        .entry("samply")
        .or_insert(samply_profile_default())
        .as_table()
        .expect("should never fail");

    let manifest = manifest_toml.to_string();

    if manifest != cargo_toml_content {
        println!("'samply' profile was added to 'Cargo.toml'");
        fs::write(&cargo_toml, manifest).expect("writing to 'Cargo.toml' failed");
    }

    // find the currently build binary name
    let mut manifest =
        cargo_toml::Manifest::from_str(cargo_toml_content).expect("failed parsing 'Cargo.toml'");

    manifest
        .complete_from_path(std::path::Path::new(&cargo_toml))
        .expect("completing manifest failed");

    let binary_name = manifest
        .bin
        .iter()
        .next()
        .expect("no binary found in 'Cargo.toml'")
        .name
        .clone()
        .expect("should never fail");

    // run cargo build with the samply profile
    // if it fails print error

    Command::new("cargo")
        .args(["build", "--profile", "samply"])
        .status()
        .expect("failed to run 'cargo build --profile samply'");

    // run samply on the binary
    // if it fails print error
    Command::new("samply")
        .args([
            "record".to_string(),
            "target/samply/".to_string() + binary_name.as_str(),
        ])
        .status()
        .expect(
            format!(
                "failed to run 'samply target/samply/{}'",
                binary_name.as_str()
            )
            .as_str(),
        );
}
