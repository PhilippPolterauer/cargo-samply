<!-- OPENSPEC:START -->
# OpenSpec Instructions

These instructions are for AI assistants working in this project.

Always open `@/openspec/AGENTS.md` when the request:
- Mentions planning or proposals (words like proposal, spec, change, plan)
- Introduces new capabilities, breaking changes, architecture shifts, or big performance/security work
- Sounds ambiguous and you need the authoritative spec before coding

Use `@/openspec/AGENTS.md` to learn:
- How to create and apply change proposals
- Spec format and conventions
- Project structure and guidelines

Keep this managed block so 'openspec update' can refresh the instructions.

<!-- OPENSPEC:END -->

# AGENTS.md (cargo-samply)

This file contains instructions for AI agents (and human contributors) working on the `cargo-samply` repository.

## 1. Build, Lint, and Test Commands

### Test
Run all tests in release mode (preferred for performance):
```bash
cargo test --release
```

Run a single unit test (by name filter):
```bash
cargo test --release <test_name_substring>
```

Run only the integration tests (CLI tests):
```bash
cargo test --release --test cli_tests
```

**Crucial:** Integration tests use `trycmd` which compares CLI output against snapshot files (`tests/*.trycmd`). If you intentionally change CLI output, you must update the snapshots:
```bash
TRYCMD=overwrite cargo test --release --test cli_tests
```

### Lint & Format
Check code formatting:
```bash
cargo fmt --all -- --check
```
*Apply formatting:* `cargo fmt`

Run Clippy (linter):
```bash
cargo clippy --all-targets --all-features
```

Run Clippy (Strict - CI fails on warnings):
```bash
cargo clippy --all-targets --all-features -- -D warnings
```

### Handy Wrappers (`justfile`)
If `just` is installed, use these shortcuts:
- `just check`: Run format check, clippy, and tests.
- `just check-strict`: Run strict clippy (deny warnings) + all checks.
- `just test`: Run tests in release mode.
- `just test-overwrite`: Update `trycmd` snapshots.

## 2. Project Structure & Testing Strategy

- **`src/`**: Main library and binary code.
  - `error.rs`: Central error handling logic using `thiserror`.
- **`tests/`**: Integration tests.
  - `*.trycmd`: Markdown files defining CLI commands and expected output. **These are the actual tests.**
  - `*.in/`: Directories containing small Cargo projects used as fixtures for the tests.
  - `cli_tests.rs`: The test harness that executes `trycmd` files.

**Agent Note:** When modifying CLI behavior, you will likely break `trycmd` tests.
1. Run `cargo test --release --test cli_tests` to see failures.
2. If the changes are correct, run `TRYCMD=overwrite ...` to update the snapshots.
3. Review the diff of `*.trycmd` files to ensure only intended changes occurred.

## 3. Code Style Guidelines

### General
- **Rust Edition:** 2021.
- **Formatting:** Keep code `rustfmt`-clean.
- **Clippy:** Keep code `clippy`-clean. Do not introduce new warnings.

### Imports
Group imports in this order, separated by empty lines:
1. `std` imports.
2. External crate imports.
3. Internal (`crate::...`) imports.

Example:
```rust
use std::path::PathBuf;

use anyhow::Context;

use crate::error::Result;
```

### Naming
- **Functions/Variables:** `snake_case`.
- **Types/Traits:** `CamelCase`.
- **Constants:** `SCREAMING_SNAKE_CASE`.

### Error Handling
- **Type:** Use `crate::error::Error` (an enum deriving `thiserror::Error`).
- **Result Alias:** Use `crate::error::Result<T>`.
- **Context:** **IMPORTANT:** When performing I/O, always add path context using the `IOResultExt` trait from `crate::error`.
  - **Do:** `fs::read_to_string(&path).path_ctx(&path)?`
  - **Don't:** `fs::read_to_string(&path)?` (lacks context on *which* file failed)
- **Avoid Panics:** Do not use `unwrap()` or `expect()` in production code. Use `?` to propagate errors.

### Logging
- Use the `log` crate macros: `debug!`, `info!`, `warn!`, `error!`.
- **Determinism:** Ensure log output is deterministic because it is captured in `trycmd` tests. Avoid printing timestamps or variable pointers in normal output.

## 4. AI Agent Guidelines

If you are an AI agent (Cursor, Copilot, Cline, etc.) operating in this repo:

1.  **Think Step-by-Step:** Before editing, analyze the file structure and read related files (`src/error.rs` is key).
2.  **Safety First:**
    - Verify your changes by running `just check` (or individual cargo commands if `just` is missing).
    - If you change CLI output, explain *why* before running the snapshot update command.
3.  **Conventions:**
    - Always strictly follow the `IOResultExt` pattern for I/O operations.
    - Check `Cargo.toml` before adding new dependencies.
4.  **Test Coverage:**
    - If adding a new feature, consider adding a new `.trycmd` file and a corresponding fixture in `tests/new_fixture.in/`.
    - If fixing a bug, verify it with a test case first.
