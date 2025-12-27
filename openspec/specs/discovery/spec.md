# discovery Specification

## Purpose
TBD - created by archiving change improve-robustness-and-ux. Update Purpose after archive.
## Requirements
### Requirement: Authoritative Target Discovery
The system SHALL use `cargo metadata` to discover targets, ensuring auto-discovered targets are found.

#### Scenario: Auto-discovered binary
- **WHEN** a project has `src/bin/tool.rs` but no `[[bin]]` in `Cargo.toml`
- **THEN** `cargo samply --bin tool` finds and runs the binary

### Requirement: Package Prioritization
When running in a workspace, the system SHALL prioritize targets in the package defined by the current working directory, unless `--package` is specified.

#### Scenario: Workspace member
- **WHEN** running inside `crates/member-a`
- **THEN** the default binary is chosen from `member-a`, not other workspace members

