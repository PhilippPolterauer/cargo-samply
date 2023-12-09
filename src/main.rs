// i want to write a programm that adds a new profile to cargo.toml if it does not exist
// if it adds a profile it should put out a warning
use cargo_toml::{DebugSetting, Manifest, Profile, Profiles};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Write;
use std::process::Command;
use std::str::from_utf8;
use toml;
use serde::Deserialize;

struct SamplyProfile {}

#[derive(Deserialize)]
struct LocateProject {
    root: String,
}

fn main() {
    // check if cargo.toml exists
    // check projct path using locate-project

    let output = Command::new("cargo")
        .arg("locate-project")
        .output()
        .expect("failed to run 'cargo locate-project'");
    dbg!(&output);
    // let result = parse(from_utf8(&output.stdout).unwrap())
    //     .expect("failed to parse output of 'cargo locate-project'");
    // parse json output for 'root' key
    // let root = ;
    let result: LocateProject = toml::from_str(from_utf8(&output.stdout).unwrap()).unwrap();
    let cargo_toml = result.root;
    println!("root: {}", cargo_toml);

    // check if cargo.toml exists
    println!("cargo.toml: {}", cargo_toml);
    // let file = File::open(root).expect("cargo.toml does not exist");
    // check if profile exists
    // if not add profile
    // if yes print warning
    let mut manifest = Manifest::from_path(&cargo_toml)
        .expect(format!("error reading manifest from '{}'", cargo_toml).as_str());
    dbg!(&manifest);

    let samply = manifest.profile.custom.get("samply");

    // let profile = if manifest.profile.custom.contains_key("samply") {
    //     manifest.profile.custom.get("samply")
    // } else {
    //     manifest.profile.custom.insert(
    //         "samply".to_owned(),
    //         Profile {
    //             debug: Some(DebugSetting::Full),
    //             build_override: None,
    //             codegen_units: None,
    //             incremental: None,
    //             lto: None,
    //             panic: None,
    //             overflow_checks: None,
    //             rpath: None,
    //             debug_assertions: None,
    //             strip: None,
    //             split_debuginfo: None,
    //             opt_level: None,
    //             package: BTreeMap::from_iter(vec![(
    //                 "include".to_owned(),
    //                 vec!["samply".to_owned()],
    //             )]),
    //         },
    //     );
    //     manifest.profile.custom.get("samply")
    // };

    // profile.unwrap().inherit_from = Some("dev".to_owned());

    let serialized = toml::to_string(&manifest).unwrap();
    // let file = File::create(cargo_toml);
    // match file {
    //     Ok(mut file) => {
    //         file.write_all(serialized.as_bytes())
    //             .expect("error writing to file");
    //     }
    //     Err(e) => println!("error writing to file: {}", e),
    // }
}
