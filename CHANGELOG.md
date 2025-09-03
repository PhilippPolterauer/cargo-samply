# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
- Version bump commands: `just bump-patch`, `just bump-minor`, `just bump-major`
- Dry run functionality: `just bump-dry LEVEL` to preview version changes

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
- Improved build automation with version management commands

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
