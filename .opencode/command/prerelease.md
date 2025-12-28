---
description: Run prerelease checks (code quality + documentation freshness).
---
Run all prerelease checks to ensure the codebase is ready for release.

**Guardrails**
- All checks must pass before a release can proceed.
- Documentation must be synchronized with current CLI behavior.
- Do not commit or push changes without user approval.

**Steps**
Track these steps as TODOs and complete them one by one.

## 1. Code Quality Checks

1.1 **Formatting** – Run `cargo fmt --all -- --check`. If it fails, run `cargo fmt --all` to fix, then re-verify.

1.2 **Linting** – Run `cargo clippy --all-targets --all-features -- -D warnings`. Fix any warnings before proceeding.

1.3 **Tests** – Run `cargo test --release`. All tests must pass.

## 2. Documentation Freshness

2.1 **CLI help vs README** – Compare the output of `cargo samply --help` against the Usage section in `README.md`. Ensure all flags, options, and environment variables are documented and match.

2.2 **Rustdoc (src/lib.rs)** – Verify the crate-level documentation in `src/lib.rs` covers:
- All target selection flags (`--bin`, `--example`, `--bench`, `--test`)
- Workspace selection (`-p/--package`)
- Samply argument forwarding (`--samply-args`)
- Benchmark flag customization (`--bench-flag`)
- Dry-run and list-targets (`--dry-run`, `--list-targets`)
- Profile injection control (`--no-profile-inject`)
- Environment variables (`CARGO_SAMPLY_SAMPLY_PATH`, `CARGO_SAMPLY_NO_PROFILE_INJECT`, `CARGO_SAMPLY_NO_SYSROOT_INJECTION`)
- Platform-specific library path variables (LD_LIBRARY_PATH, DYLD_LIBRARY_PATH, PATH)

2.3 **OpenSpec specs** – Check that `openspec/specs/*/spec.md` files have valid Purpose sections (not "TBD"). If any are outdated, update them.

2.4 **CHANGELOG** – Verify `CHANGELOG.md` has an entry for the upcoming version with all notable changes documented.

## 3. Final Validation

3.1 Run `cargo doc --no-deps` to ensure rustdoc builds without warnings.

3.2 If the project uses OpenSpec, run `openspec validate --strict` to ensure all specs are valid.

3.3 Summarize any issues found and fixes applied. If all checks pass, report "Prerelease checks passed."

**Reference**
- See `AGENTS.md` for coding conventions and error handling patterns.
- See `justfile` for convenient command shortcuts (`just check`, `just check-strict`).
- See `openspec/project.md` for project context and testing strategy.
