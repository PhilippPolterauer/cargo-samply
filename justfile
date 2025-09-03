# Justfile for cargo-samply

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
