use std::env;
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let raw_args: Vec<String> = env::args().skip(1).collect();
    println!("fake-samply raw args={raw_args:?}");

    let mut profiler_args = Vec::new();
    let mut runtime_args = Vec::new();
    let mut after_separator = false;

    for arg in &raw_args {
        if !after_separator && arg == "--" {
            after_separator = true;
            continue;
        }
        if after_separator {
            runtime_args.push(arg.clone());
        } else {
            profiler_args.push(arg.clone());
        }
    }

    println!("fake-samply prefix={profiler_args:?}");
    println!("fake-samply runtime={runtime_args:?}");

    if runtime_args.is_empty() {
        eprintln!("fake-samply: missing profiled command");
        return Err("no command after --".into());
    }

    let status = Command::new(&runtime_args[0])
        .args(&runtime_args[1..])
        .status()?;

    if !status.success() {
        return Err(format!("profiled command exited with {status}").into());
    }

    Ok(())
}
