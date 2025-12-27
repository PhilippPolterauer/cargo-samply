# Project Context

## Purpose
`cargo-samply` is a Cargo subcommand (`cargo samply`) that automates the usual Rust profiling workflow with the external [`samply`](https://github.com/mstange/samply) profiler.

What it does:
- Builds a selected Cargo target (bin/example/bench) with a chosen Cargo profile (default: `samply`).
- Ensures the `[profile.samply]` section exists in the project’s `Cargo.toml` (inherits `release`, sets `debug = true`).
- Runs the built artifact under `samply` (`samply record -- <artifact> ...`).

Non-goals / scope notes:
- This project is an independent Cargo subcommand and is not affiliated with upstream `samply`.
- Benchmark profiling behavior is primarily validated with Criterion-style benches.

## Tech Stack
- Language: Rust (Edition 2021)
- CLI parsing: `clap` (derive)
- Cargo introspection:
  - `cargo_metadata` (parse `cargo build` JSON messages)
  - `cargo_toml` (read manifest for bins/benches/default-run)
  - `toml` (read/modify manifest content in some paths)
- Error handling: `thiserror` (`src/error.rs`)
- Logging: `log` + `ocli`
- Testing:
  - Unit tests via `cargo test` (in `src/main.rs` and `src/util.rs`)
  - Snapshot/CLI tests via `trycmd` (`tests/*.trycmd`)
- CI: GitHub Actions on Ubuntu/Windows/macOS (`.github/workflows/ci.yml`)

## Project Conventions

### Code Style
- Formatting: `rustfmt` (CI runs `cargo fmt --all -- --check`)
- Linting: `clippy` (CI runs `cargo clippy --all-targets --all-features`)
- Naming: standard Rust conventions (`snake_case`, `CamelCase`, `SCREAMING_SNAKE_CASE`)
- Imports: group in this order with a blank line between groups:
  1. `std::...`
  2. external crates
  3. `crate::...`
- Error handling:
  - Prefer `crate::error::{Error, Result}`.
  - Avoid panics (`unwrap`/`expect`) in production code.
  - When performing I/O, add path context using `crate::error::IOResultExt::path_ctx`.
- Logs / output determinism:
  - Prefer `debug!` / `info!` / `warn!` / `error!`.
  - Keep CLI output deterministic because `trycmd` snapshots assert on output.

### Architecture Patterns
- High-level layout:
  - `src/main.rs`: orchestration (argument handling, running `cargo build`, launching `samply`)
  - `src/cli.rs`: CLI schema (`CargoCli` and `Config`)
  - `src/util.rs`: helper functions (locate project, ensure profile, target resolution, env var injection)
  - `src/error.rs`: centralized error enum + `Result` alias + `IOResultExt`
- Subprocess-heavy tool:
  - Uses `cargo locate-project`, `cargo build` (JSON message format), and `rustc` queries.
  - Uses `util::CommandExt` (`.log()` + `.call()`) for consistent logging/exec behavior.
- Cross-platform behavior is first-class (Windows `.exe`, env var differences, path separators).

### Testing Strategy
- Unit tests:
  - Co-located `mod tests` blocks for internal logic.
- Integration/CLI tests:
  - Authoritative CLI behavior is tested via `trycmd` snapshots (`tests/*.trycmd`).
  - Test fixtures are small Cargo projects under `tests/*.in/`.
  - A test-only `fake-samply` binary exists at `tests/bin/fake_samply.rs`.
  - The harness sets `CARGO_SAMPLY_SAMPLY_PATH` to the fake binary (`tests/cli_tests.rs`).
- Commands:
  - Run everything like CI: `cargo test --release` (or `just test`).
  - Run only CLI tests: `cargo test --release --test cli_tests`.
  - Update snapshots (only if output change is intentional): `TRYCMD=overwrite cargo test --release --test cli_tests` (or `just test-overwrite`).

### Git Workflow
- Standard PR-based workflow (feature branch → PR).
- Before merging, ensure:
  - Tests pass (`cargo test --release`)
  - Formatting and Clippy are clean (`cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features`)
- Commit messages in this repo are short and imperative (examples: “Add …”, “Refactor …”, “Update …”).

## Domain Context
- Cargo subcommand behavior:
  - Must behave correctly when invoked as `cargo samply ...` and as `cargo-samply ...`.
- Target selection rules:
  - Exactly one of `--bin`, `--example`, `--bench` may be specified.
  - If none is specified, a default binary is inferred (via `default-run`, local manifest targets, or workspace metadata).
- Benchmark profiling:
  - For `--bench`, runtime args inject `--bench` (mirrors `cargo bench` behavior). This is primarily validated for Criterion harnesses.
- Samply profile management:
  - The `[profile.samply]` section is central to the tool’s purpose and is created automatically when missing.

## Important Constraints
- Cross-platform (Linux/macOS/Windows) and CI coverage across all three.
- Output determinism:
  - CI sets `NO_COLOR=1`, `TERM=dumb`, `CARGO_TERM_COLOR=never`.
  - Avoid introducing new non-deterministic output that would break `trycmd` snapshots.
- Avoid unnecessary dependencies; keep the tool lightweight.
- Be careful modifying user projects:
  - `ensure_samply_profile` appends to `Cargo.toml`; changes should be minimal and safe.

## External Dependencies
- External CLIs:
  - `cargo` (build system and metadata)
  - `rustc` (sysroot / host target queries)
  - `samply` (profiler) 
- Environment variables:
  - `CARGO_SAMPLY_SAMPLY_PATH`: override the `samply` executable path (used by tests).
  - `CARGO_SAMPLY_NO_SYSROOT_INJECTION`: disables sysroot library-path injection (primarily for debugging/tests).
- Platform-specific runtime env vars:
  - Linux: `LD_LIBRARY_PATH`
  - macOS: `DYLD_LIBRARY_PATH`
  - Windows: `PATH`
