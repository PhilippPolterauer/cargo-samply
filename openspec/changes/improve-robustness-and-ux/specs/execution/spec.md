## ADDED Requirements

### Requirement: Build-Driven Path Resolution
The system SHALL determine the binary path by parsing the `CompilerArtifact` output from `cargo build`, rather than predicting the path.

#### Scenario: Cross-compilation
- **WHEN** building for a different target triple
- **THEN** the correct artifact path from the build output is used

### Requirement: Proactive Tool Check
The system SHALL check for the existence of `samply` in the system PATH before starting the build.

#### Scenario: Samply missing
- **WHEN** `samply` is not installed
- **THEN** the tool exits with a clear error before compiling the project
