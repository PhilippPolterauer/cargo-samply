# cargo-samply

[![Continuous integration](https://github.com/PhilippPolterauer/cargo-samply/actions/workflows/ci.yml/badge.svg)](https://github.com/PhilippPolterauer/cargo-samply/actions/workflows/ci.yml)
[![docs.rs](https://img.shields.io/docsrs/cargo-samply/latest)](https://docs.rs/cargo-samply)
![GitHub License](https://img.shields.io/github/license/PhilippPolterauer/cargo-samply?style=flat&link=https%3A%2F%2Fgithub.com%2FPhilippPolterauer%2Fcargo-samply%3Ftab%3DMIT-1-ov-file)

Profile Rust code with `samply`, without remembering the build/run ceremony.

> **Important:** This project is **not affiliated with, endorsed by, or maintained by** the official `samply` project or its authors. It is an independent Cargo subcommand that happens to integrate with the `samply` profiler.

## What it does

`cargo samply` automates the usual profiling workflow:

- Builds your project (optionally with a custom Cargo profile)
- Runs the resulting artifact under the `samply` profiler
- Supports binaries, examples, and benchmark targets

### Why use it?

- **Automated "Profile Ceremony"**: It handles the `[profile.samply]` management for you, ensuring optimized code with debug symbols without manual `Cargo.toml` edits.
- **Unified Target Selection**: No more hunting for compiled artifacts in `target/release/examples/...`. Just use `--bin`, `--example`, or `--bench`.
- **Smart Benchmark Handling**: Automatically injects the correct runtime flags (like `--bench`) for Criterion-style benchmarks, which are often missed when profiling manually.
- **Lower Friction**: By making profiling a single command, it encourages frequent performance checks throughout development.

## Installation

### Prerequisites

- A working Rust toolchain (via `rustup`)
- `samply` installed and available in your `PATH`

### Install from crates.io

```bash
cargo install cargo-samply
cargo install samply
```

### Install from git

```bash
cargo install --git https://github.com/PhilippPolterauer/cargo-samply.git
```

## Quickstart

From any Rust project:

```console
$ cargo samply
```

Once `samply` starts its local UI server, open the printed address (typically `127.0.0.1:3001`).

![Samply Web View](https://raw.githubusercontent.com/PhilippPolterauer/cargo-samply/main/doc/samply-web.png)

## Usage

```console
$ cargo samply --help
The samply subcommand

Usage: cargo samply [OPTIONS] [TRAILING_ARGUMENTS]...

Arguments:
  [TRAILING_ARGUMENTS]...  Trailing arguments passed to the binary being profiled

Options:
  -p, --profile <PROFILE>    Build with the specified profile [default: samply]
  -b, --bin <BIN>            Binary to run
  -e, --example <EXAMPLE>    Example to run
      --bench <BENCH>        Benchmark target to run
  -f, --features <FEATURES>  Build features to enable
      --no-default-features  Disable default features
  -v, --verbose              Print extra output to help debug problems
  -q, --quiet                Suppress all output except errors
  -n, --no-samply            Disable the automatic samply start
  -h, --help                 Print help
  -V, --version              Print version
```

## Common recipes

```bash
# Profile the default binary target
cargo samply

# Profile a specific binary
cargo samply --bin my-binary

# Profile an example
cargo samply --example my-example

# Profile a benchmark (Criterion harness validated)
cargo samply --bench throughput -- --sample-size 10

# Build using a different profile
cargo samply --profile release

# Enable specific features
cargo samply --features feature1,feature2

# Disable default features
cargo samply --no-default-features

# Pass arguments to the program being profiled
cargo samply -- arg1 arg2 --flag value

# Run without starting samply (useful for debugging build/target selection)
cargo samply --no-samply
```

### Notes on benchmarks

- When you use `--bench <name>`, `cargo-samply` prefixes the runtime invocation with `--bench` (mirroring `cargo bench`).
- This behavior has been validated with Criterion-driven benches only; other harnesses/runners may require manual adjustments.
- Benchmark targets must be referenced by their exact Cargo target name (no suffix rewriting or aliasing is performed).

## Development

This project includes a `justfile` for common development tasks. Install [just](https://github.com/casey/just) and use:

```bash
# Run tests (matches CI configuration)
just test

# Update test snapshots when needed
just test-overwrite

# Clean all target directories
just clean-all

# Clean only test project target directories
just clean

# Clean only main project
just clean-main
```

### Testing

The project uses `trycmd` for integration testing, which validates CLI behavior against snapshot files.
When making changes that affect command output:

1. Run `just test`
2. If output changed intentionally, run `just test-overwrite`
3. Review the snapshot diffs in git

## Contributing

Issues and PRs are welcome.

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run `just test`
5. Open a pull request
