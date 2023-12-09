use serde::Deserialize;
use std::fs;
use std::process::{Command, Output};
use std::str::{from_utf8, FromStr};

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

fn run_command(command: &str, args: Vec<&str>) -> Output {
    let mut commandvec = vec![command];
    for arg in &args {
        commandvec.push(arg);
    }
    let commandstr = &commandvec.join(" ");
    println!("running '{}'", &commandstr);
    let output = Command::new(command)
        .args(args)
        .output()
        .expect(format!("failed to run '{}'", commandstr).as_str());

    if !output.status.success() {
        if let Some(code) = &output.status.code() {
            println!("'{}' failed with code '{}'", commandstr, code);
            std::process::exit(*code);
        }
    }
    output
}

fn main() {
    // check if cargo.toml exists
    // check project path using locate-project

    let output = run_command("cargo", vec!["locate-project"]);
    let result: Result<LocateProject, serde_json::Error> =
        serde_json::from_str(from_utf8(&output.stdout).unwrap());

    let cargo_toml: String;
    if let Ok(result) = result {
        cargo_toml = result.root;
    } else {
        println!("cargo locate-project: failed");
        std::process::exit(1);
    }

    // check if cargo.toml exists
    println!("cargo.toml: {}", cargo_toml);
    // let file = File::open(root).expect("cargo.toml does not exist");
    // check if profile exists
    // if not add profile
    // if yes print warning

    let binding: String = fs::read_to_string(&cargo_toml)
        .unwrap_or_else(|_| panic!("failed reading '{}'", &cargo_toml));
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

    // first we find the available binaries
    let binaries = manifest.bin;
    if binaries.is_empty() {
        println!("no binary found in 'Cargo.toml'");
        std::process::exit(1);
    }
    // if length equal to one then we use it
    let def_binary_name = if binaries.len() == 1 {
        binaries.first().unwrap().name.clone().unwrap()
    } else {
        let name = manifest
            .package
            .expect("no package section")
            .default_run
            .expect("no default-run specified");

        // we look for the binary with the correct name and error if we can't find it
        if let Some(binary) = binaries.iter().find_map(|s| {
            if let Some(n) = &s.name {
                if n == &name {
                    s.name.clone()
                } else {
                    None
                }
            } else {
                None
            }
        }) {
            binary
        } else {
            println!("did not find binary '{}'", name);
            std::process::exit(1);
        }
    };

    // parse additional arguments from the command line
    let args: Vec<String> = std::env::args().collect();
    let mut build_args: Vec<&str> = vec!["build", "--profile", "samply"];
    let binary_name =
        if let Some((idx, _)) = args.iter().enumerate().find(|(_, s)| s.as_str() == "--bin") {
            if let Some(arg) = args.get(idx + 1).clone() {
                build_args.push("--bin");
                build_args.push(args[idx + 1].as_str());
                "target/samply/".to_string() + &arg
            } else {
                println!("--bin requires an argument");
                std::process::exit(1);
            }
        } else if let Some((idx, _)) = args
            .iter()
            .enumerate()
            .find(|(_, s)| s.as_str() == "--example")
        {
            let arg = args[idx + 1].clone();

            build_args.push("--example");
            build_args.push(args[idx + 1].as_str());
            "target/samply/examples/".to_string() + &arg
        } else {
            "target/samply/".to_string() + &def_binary_name
        };

    // run cargo build with the samply profile
    // if it fails print error
    run_command("cargo", build_args);
    // run samply on the binary
    // if it fails print error
    run_command("samply", vec!["record", &binary_name]);
}
