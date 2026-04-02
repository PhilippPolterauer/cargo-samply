---
description: Create a git tag and trigger the automated release workflow.
argument-hint: [version]
---
Perform the release process for the specified version.

**Guardrails**
- The working directory must be clean (no uncommitted changes).
- The version in `Cargo.toml` must match the requested version.
- Prerelease checks (`/prerelease`) should ideally have been run before this (remind the user).
- STOP and ask for confirmation before creating or pushing tags.

**Steps**
Track these steps as TODOs and complete them one by one.

## 1. Validation
1.1 **Version Check** – Read `Cargo.toml` to verify the `version` field matches the argument provided by the user. If they differ, abort and ask the user to update `Cargo.toml` or provide the correct version.
1.2 **Git Status** – Run `git status --porcelain`. If output is not empty, abort and ask user to commit or stash changes.

## 2. Trigger Automated Release
2.1 **Create Git Tag** – Run `git tag v<VERSION>`.
2.2 **Push Tag** – Run `git push origin v<VERSION>`.
2.3 **Monitor Release Workflow** – Watch `.github/workflows/release.yml` to ensure it verifies the tag, runs lint/build/tests, publishes to crates.io, and creates the GitHub release.
2.4 **Confirm Results** – Verify the workflow succeeded, the crate version is available on crates.io, and the GitHub release exists.

**Reference**
- The release workflow expects a `CARGO_REGISTRY_TOKEN` repository secret for crates.io publishing.
- If the publish step succeeds but a later step fails, re-run only the failed jobs instead of re-running the publish job.
