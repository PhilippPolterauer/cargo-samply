# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
- `--quiet` flag to suppress all output except errors

### Changed
- Default log level changed from `Info` to `Warn` to reduce verbose output
- `features` argument now accepts multiple values instead of a single string
- Improved error handling with proper error propagation in `util.rs`
- Enhanced CLI argument validation and user experience

### Removed
- Automatic installation of `samply` in build script (now requires manual installation)

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
