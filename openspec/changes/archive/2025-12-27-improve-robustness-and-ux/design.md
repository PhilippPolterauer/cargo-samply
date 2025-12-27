## Context
`cargo-samply` currently uses a mix of `cargo_toml` (manual parsing) and `cargo_metadata` for target discovery. It also "guesses" where binaries will be output based on profile strings. This is fragile.

## Goals
- **Single Source of Truth**: Use `cargo` (via metadata and build output) as the authority on targets and paths.
- **Cargo Alignment**: Flags and behavior should mirror `cargo` where possible.
- **Shell Safety**: Output commands must be safe to execute.

## Decisions

### 1. Artifact Resolution via Build Output
Instead of `resolve_target_path` guessing `target/release/examples/foo`, we will capture the JSON stream from `cargo build --message-format=json`.
- **Pros**: Handles cross-compilation, `CARGO_TARGET_DIR`, workspace layouts, and filename variations (e.g. Windows `.exe`) automatically.
- **Cons**: Requires parsing the stream during the build phase. The current "plan then execute" model needs adjustment to "plan build, execute build, parse result, plan run, execute run".

### 2. Metadata-only Discovery
We will drop `cargo_toml` manual parsing for target discovery and rely solely on `cargo metadata --no-deps`.
- **Pros**: Correctly identifies auto-discovered binaries (`src/bin/*.rs`) and tests.

### 3. CLI Flag Standardization
- `-p` becomes `--package`.
- `--profile` loses the short flag (or gets a different one, but none is standard in cargo for profile).
- This is a breaking change but necessary for consistency.

## Risks / Trade-offs
- **Breaking Change**: Users used to `cargo samply -p release` will face errors. We should provide a clear error message if `-p` is passed a value that looks like a profile but is not a package in the workspace (though hard to distinguish reliably).
- **Complexity**: parsing JSON output is slightly more complex than blind execution, but `cargo_metadata` crate handles the types.

## Migration Plan
- Update `README.md` to reflect flag changes.
- Bump version to `0.5.0` (semver minor bump for breaking change in pre-1.0).
