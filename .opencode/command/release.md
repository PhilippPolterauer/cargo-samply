---
description: Create GitHub release, git tag, and publish to crates.io.
argument-hint: [version]
---
Perform the release process for the specified version.

**Guardrails**
- The working directory must be clean (no uncommitted changes).
- The version in `Cargo.toml` must match the requested version.
- Prerelease checks (`/prerelease`) should ideally have been run before this (remind the user).
- STOP and ask for confirmation before running `cargo publish`, creating releases, or pushing tags.

**Steps**
Track these steps as TODOs and complete them one by one.

## 1. Validation
1.1 **Version Check** – Read `Cargo.toml` to verify the `version` field matches the argument provided by the user. If they differ, abort and ask the user to update `Cargo.toml` or provide the correct version.
1.2 **Git Status** – Run `git status --porcelain`. If output is not empty, abort and ask user to commit or stash changes.

## 2. Release & Publish
2.1 **Create Git Tag** – Run `git tag v<VERSION>`.
2.2 **Push Tag** – Run `git push origin v<VERSION>`.
2.3 **Create GitHub Release** – Run `gh release create v<VERSION> --generate-notes`.
2.4 **Publish to Crates.io** – Run `cargo publish`.

**Reference**
- Ensure you have the `gh` CLI and `cargo` installed and authenticated.
- If `cargo publish` fails, the git tag and GitHub release will already exist. This is usually acceptable, but check `cargo publish --dry-run` first if you want to be extra safe (optional).
