## 1. CLI Refactoring
- [x] 1.1 Update `src/cli.rs` to rename `profile` short flag and add `package` arg.
- [x] 1.2 Add `--test` and `--samply-args` fields to `Config`.
- [x] 1.3 Update tests to reflect CLI changes.

## 2. Discovery Improvements
- [x] 2.1 Refactor `util::get_all_targets` to use `cargo_metadata` exclusively.
- [x] 2.2 Implement logic to filter targets by the selected `--package` (or current CWD package).
- [x] 2.3 Add support for discovering test targets.

## 3. Execution Refactoring
- [x] 3.1 Refactor `main.rs` to split planning into "Build Plan" and "Run Plan".
- [x] 3.2 Implement `CompilerArtifact` parsing to extract the executable path.
- [x] 3.3 Ensure `run` logic uses the extracted path.
- [x] 3.4 Add `which` check for `samply` in the planning phase.

## 4. UX & Output
- [x] 4.1 Implement `shell-words` or similar quoting for `print_plan`.
- [x] 4.2 Format dry-run output as `VAR=val cmd args...`.
- [x] 4.3 Verify dry-run output with a test case containing spaces in paths.

## 5. Testing & Cleanup
- [x] 5.1 Add integration test for `--test` target.
- [x] 5.2 Add integration test for `--samply-args`.
- [x] 5.3 Update `README.md` and `CHANGELOG.md`.
