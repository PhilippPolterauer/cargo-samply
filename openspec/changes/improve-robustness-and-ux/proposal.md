# Change: Improve Robustness and User Experience

## Why
Analysis of the current codebase revealed several issues affecting robustness and user experience:
- Target discovery relies on manual `Cargo.toml` parsing, which misses auto-discovered targets.
- Binary path resolution predicts paths based on conventions, which fails with non-standard target directories or cross-compilation.
- The `-p` flag targets a profile, whereas in `cargo` it universally means `--package`.
- `--dry-run` output is not shell-compatible (spaces are not quoted), making it hard to debug or reuse commands.
- Users cannot profile integration tests or pass flags to `samply` itself.

## What Changes
- **BREAKING**: The `-p` short flag is removed from `--profile` and reassigned to `--package` to align with Cargo conventions.
- **Robust Discovery**: Switch to `cargo metadata` for authoritative target discovery.
- **Robust Execution**: Parse `cargo build` JSON output (`CompilerArtifact`) to find the exact binary path instead of predicting it.
- **UX**: `--dry-run` now emits properly quoted, copy-pasteable shell commands.
- **Features**: 
  - Add `--test <NAME>` support.
  - Allow passing arguments to `samply` (e.g., `cargo samply --samply-args "--rate 2000" ...`).
  - Proactively check for `samply` installation.

## Impact
- **Affected Specs**: `cli`, `execution`, `discovery`.
- **Affected Code**: `src/main.rs`, `src/util.rs`, `src/cli.rs`.
- **Migration**: Users relying on `cargo samply -p release` will need to use `cargo samply --profile release` or `cargo samply -p <package>` (if they meant package).
