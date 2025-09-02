// build.rs

fn main() {
    // check if 'samply' binary is installed in the system
    let samply_available = std::process::Command::new("samply")
        .arg("--help")
        .status()
        .is_ok();

    if !samply_available {
        println!("cargo:warning='samply' is not installed. Please run `cargo install samply` and ensure it is in your PATH.");
        // Optionally, you can fail the build:
        // std::process::exit(1);
    }
    println!("cargo:rerun-if-changed=build.rs");
}
