# Justfile for cargo-samply

# Run all checks (test, clippy, format)
check:
    #!/usr/bin/env bash
    echo "🔍 Running all checks..."
    echo ""
    echo "📝 Checking formatting..."
    cargo fmt --check
    echo "✅ Formatting check passed"
    echo ""
    echo "📎 Running clippy..."
    cargo clippy --all-targets --all-features
    echo "✅ Clippy check passed"
    echo ""
    echo "🧪 Running tests..."
    cargo test --release
    echo "✅ All tests passed"
    echo ""
    echo "🎉 All checks passed successfully!"

# Run strict checks (test, clippy with deny warnings, format)
check-strict:
    #!/usr/bin/env bash
    echo "🔍 Running strict checks..."
    echo ""
    echo "📝 Checking formatting..."
    cargo fmt --check
    echo "✅ Formatting check passed"
    echo ""
    echo "📎 Running clippy (strict)..."
    cargo clippy --all-targets --all-features -- -D warnings
    echo "✅ Clippy strict check passed"
    echo ""
    echo "🧪 Running tests..."
    cargo test --release
    echo "✅ All tests passed"
    echo ""
    echo "🎉 All strict checks passed successfully!"

# Clean all target directories from test cargo projects
clean:
    #!/usr/bin/env bash
    echo "Cleaning target directories from test projects..."
    find tests/ -name "target" -type d -exec rm -rf {} + 2>/dev/null || true
    echo "Done cleaning test target directories."

# Clean main project target directory
clean-main:
    cargo clean

# Clean everything (main project + test projects)
clean-all: clean clean-main
    echo "All target directories cleaned."

# Run tests (matches CI configuration)
test:
     cargo test --release

# Run tests with trycmd overwrite (for updating test snapshots)
test-overwrite:
    TRYCMD=overwrite cargo test --release
