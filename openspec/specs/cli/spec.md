# cli Specification

## Purpose
TBD - created by archiving change improve-robustness-and-ux. Update Purpose after archive.
## Requirements
### Requirement: Standardized Cargo Flags
The CLI SHALL align with standard Cargo conventions.
- The `-p` flag MUST refer to `--package`.
- The `--profile` flag MUST NOT use `-p` as a short flag.

#### Scenario: User selects a package
- **WHEN** user runs `cargo samply -p my-pkg`
- **THEN** it selects the binary from `my-pkg`

#### Scenario: User selects a profile
- **WHEN** user runs `cargo samply --profile release`
- **THEN** it builds with the release profile

### Requirement: Profiling Integration Tests
The CLI SHALL support profiling integration tests via a `--test` flag.

#### Scenario: User profiles a test
- **WHEN** user runs `cargo samply --test integration_suite`
- **THEN** the test binary is built and profiled

### Requirement: Samply Argument Forwarding
The CLI SHALL allow passing arguments directly to the `samply` process.

#### Scenario: User sets sample rate
- **WHEN** user runs `cargo samply --samply-args "--rate 2000" --bin app`
- **THEN** `samply` is invoked with `--rate 2000`

### Requirement: Shell-Compatible Dry-Run
The `--dry-run` output SHALL be a valid, copy-pasteable shell command.
- Arguments containing spaces MUST be quoted.
- Environment variables MUST be prepended as `KEY=VALUE`.

#### Scenario: Path with spaces
- **WHEN** the project path contains "My Project"
- **THEN** the dry-run output quotes the path: `samply record -- "/path/to/My Project/target/..."`

