# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.1] - 2025-12-28

### Documentation

- Updated crate-level documentation (`src/lib.rs`) to fully match the README, including new flags like `--test`, `--bench-flag`, and `--dry-run`.
- Updated OpenSpec purpose descriptions to be valid.

## [0.4.0] - 2025-12-28

### Added

- `--test <NAME>` flag to profile integration tests and test binaries.
- `--samply-args <ARGS>` flag to pass arguments directly to `samply` (e.g., `--samply-args "--rate 2000"`).
- `-p, --package <PKG>` flag to select a package in a workspace (aligns with standard Cargo conventions).
- Proactive `samply` installation check before starting the build, providing earlier feedback if `samply` is missing.
- Test targets are now discoverable via `--list-targets`.
- `--dry-run` flag to preview build and run commands without executing them.
- `--no-profile-inject` flag and `CARGO_SAMPLY_NO_PROFILE_INJECT` environment variable to prevent automatic `Cargo.toml` modification.
- `--bench-flag <FLAG>` option to customize the flag injected when running benchmark targets (default: `--bench`). Use `--bench-flag=none` to disable injection entirely.
- `--list-targets` flag to list all available binaries, examples, benchmarks, and tests in the workspace.
- Support for `CARGO_SAMPLY_NO_SYSROOT_INJECTION` environment variable to disable sysroot injection.
- New integration test for dynamic linking scenarios in `tests/dylib.in`.
- Target-specific `rustlib` directory support in library path configuration.

### Changed

- **BREAKING**: The `-p` short flag is now `--package` (matching Cargo conventions). Use `--profile` (no short flag) for build profile selection.
- Target discovery now uses `cargo metadata` exclusively for authoritative and complete target enumeration, including auto-discovered targets (like `src/bin/*.rs`).
- `--dry-run` output now uses proper shell quoting (via `shell-words`) and prepends environment variables as `VAR=val`, making the output safe to copy-paste.
- Updated README with documentation for all new CLI flags and environment variables.

### Fixed

- Fixed dynamic library path resolution for binaries with Rust dylib dependencies (e.g., projects using `prefer-dynamic` or Bevy with `dynamic_linking`).
- Added automatic detection of Rust sysroot and injection of appropriate library paths (`DYLD_LIBRARY_PATH` on macOS, `LD_LIBRARY_PATH` on Linux, `PATH` on Windows) when spawning profiled binaries.
- Improved error message for rustc sysroot detection failure.
- Improved host target detection by querying `rustc` instead of using hardcoded values.
- Improved binary path detection by parsing `cargo build` output, enabling support for cross-compilation and custom target directories.
- Improved target triple detection for library path configuration by inferring it from the binary path, falling back to the host target.
- Replaced generic `io::Error::other` with specific error variants (`InvalidSamplyArgs`, `CargoStdoutCaptureFailed`, `CargoMetadataFailed`) for better error handling.
- Fixed CI code coverage failures by disabling ANSI color output in coverage jobs.

### Internal

- Added thread-safe environment variable testing using a global mutex to prevent flaky test failures in CI.
- Optimized target discovery by avoiding redundant `cargo metadata` calls during target resolution.
- Removed compile-time check for `samply` in `build.rs` to allow installation without `samply` pre-installed.
- Refactored `src/main.rs` by extracting helper functions to reduce cyclomatic complexity.
- Refactored `src/util.rs` to deduplicate TOML parsing logic in profile checks.
- Added `.rustfmt.toml` configuration to ensure consistent code style.
- Enhanced CI infrastructure with security auditing (`cargo-audit`), code coverage (`cargo-llvm-cov`), and automated dependency updates (Dependabot).
- Optimized CI workflows by splitting lint and test jobs and enabling caching for faster feedback.

### Documentation

- Updated library overview documentation to include test profiling support and usage examples.
- Added missing API documentation for public items in the `util` module.


## [0.3.4] - 2025-12-12

### Added

- Support for profiling benchmark targets via the new `--bench` flag (validated with Criterion harnesses), including automatic discovery of Criterion/custom harness binaries
- Test-only `fake-samply` shim plus integration coverage to ensure `samply record -- <command>` invocations stay correct
- Automatically forward the `--bench` flag to benchmark executables (matching `cargo bench`)

### Changed

- Improved error messaging for conflicting target-selection flags to mention benchmark support
- Bench names must now match their exact Cargo target names (no `_bench`/`-bench` aliasing)

## [0.3.2] - 2025-09-03

### Fixed

- upgraded all packages to be latest versions
- License text and README formatting

## [0.3.1] - 2025-09-03

### Added

- `justfile` with useful development commands:
  - `just clean` - Remove target directories from test projects
  - `just clean-main` - Clean main project target directory
  - `just clean-all` - Clean all target directories
  - `just test` - Run tests matching CI configuration
  - `just test-overwrite` - Update test snapshots
  - `just check` - Run all checks (formatting, clippy, tests)
  - `just check-strict` - Run strict checks with warnings as errors
- Enhanced bin guessing logic with workspace support using `cargo_metadata`
- Improved error messages with actionable suggestions and copy-pasteable commands
- Comprehensive workspace detection for binaries and examples

### Fixed

- Fixed CI test failures by removing ANSI color codes from test expectations
- Improved test stability by cleaning up colored output in test snapshots
- Fixed integration test consistency issues
- Enhanced workspace metadata handling with proper fallback for test scenarios
- Improved error messages for complex workspace scenarios

### Changed

- Converted metadata function to use `cargo_metadata` crate for better type safety
- Enhanced error handling with structured suggestions for available binaries and examples
- Updated test expectations to match improved error message format

### DevOps

- Added justfile for standardized development workflows
- Improved test command consistency between local development and CI
- Enhanced development experience with automated cleanup commands
- Added comprehensive test coverage for workspace scenarios

## [0.3.0] - 2025-09-03

### Added

- `--quiet` flag to suppress all output except errors
- Comprehensive documentation with usage examples and API docs
- New "Passing Arguments to the Binary" documentation section
- Extensive unit test coverage for all modules
- Integration tests using `trycmd` for CLI behavior validation
- Library structure with proper module exports for `cargo-samply` as a library
- Helper functions: `get_bin_path()` for binary path resolution
- Support for both direct execution and cargo subcommand protocols
- Detailed error documentation with examples
- Comprehensive rustdoc documentation for all public APIs

### Changed

- Default log level changed from `Info` to `Warn` to reduce verbose output
- `features` argument now accepts multiple values (`Vec<String>`) instead of a single string
- Improved error handling with proper error propagation throughout the codebase
- Enhanced CLI argument validation and user experience
- Refactored main.rs with dual CLI parsing support (direct vs cargo subcommand)
- Updated project structure to support both binary and library usage
- Improved argument passing documentation with practical examples
- Enhanced error messages with better context and suggestions

### Fixed

- Fixed cargo subcommand protocol compliance for proper `cargo samply` integration
- Replaced all `unwrap()` calls with proper error handling
- Fixed lifetime issues in argument handling
- Corrected binary existence checks before execution
- Resolved integration test stability with proper output wildcards

### Removed

- Automatic installation of `samply` in build script (now requires manual installation)
- Debug output by default (now only shown with `--verbose` flag)

### Documentation

- Complete API documentation for all modules and functions
- Usage examples for all major features
- Installation and setup instructions
- Detailed explanation of the `samply` profile management
- Examples of passing arguments to binaries under test

### DevOps

- Updated CI workflow to run tests on push and pull requests
- Added comprehensive test suite with 11 unit tests and 6 integration tests
- Improved test coverage across all modules
- Added `tempfile` dependency for unit testing
- Enhanced release preparation with proper versioning

## [0.2.0] - 2025-09-02

### Added

- Support for multiple features via `--features` (comma-separated)
- Availability check for `samply` binary before execution
- Comprehensive unit tests for utility functions
- `SamplyNotFound` error variant for better user feedback

### Fixed

- Replaced `unwrap()` calls with proper error handling
- Fixed lifetime issues in argument handling
- Corrected binary existence checks before execution

### DevOps

- Updated CI workflow to run tests on push and pull requests
- Added `tempfile` dependency for unit testing
- Improved test coverage with both unit and integration tests

## [0.1.0] - Initial Release

- Basic functionality for running `samply` on Cargo binaries
- Support for profiling binaries and examples
- Integration with Cargo build system
