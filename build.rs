// build.rs

fn main() {
    // check if 'samply' binary is installed in the system
    let samply_available = std::process::Command::new("samply")
        .arg("--help")
        .status()
        .is_ok();

    if !samply_available {
        // Install another package using Cargo
        println!("cargo:warning=install samply");
        let status = std::process::Command::new("cargo")
            .args(["install", "samply"])
            .status()
            .expect("Failed to run cargo install");

        if !status.success() {
            eprintln!("Failed to install another_package_name");
            std::process::exit(1);
        }
        println!("cargo:warning=install samply done!");
    }
    println!("cargo:rerun-if-changed=build.rs");
}
