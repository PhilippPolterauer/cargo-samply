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

# Bump patch version (e.g., 1.2.3 -> 1.2.4)
bump-patch:
    #!/usr/bin/env bash
    echo "🔖 Bumping patch version..."
    just _bump patch

# Bump minor version (e.g., 1.2.3 -> 1.3.0)
bump-minor:
    #!/usr/bin/env bash
    echo "🔖 Bumping minor version..."
    just _bump minor

# Bump major version (e.g., 1.2.3 -> 2.0.0)
bump-major:
    #!/usr/bin/env bash
    echo "🔖 Bumping major version..."
    just _bump major

# Internal command to handle version bumping
_bump LEVEL:
    #!/usr/bin/env bash
    set -euo pipefail
    
    # Check if working directory is clean
    if [[ -n $(git status --porcelain) ]]; then
        echo "❌ Error: Working directory is not clean. Please commit or stash changes first."
        exit 1
    fi
    
    # Get current version from Cargo.toml
    CURRENT_VERSION=$(grep '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/')
    echo "📝 Current version: $CURRENT_VERSION"
    
    # Parse version components
    IFS='.' read -r MAJOR MINOR PATCH <<< "$CURRENT_VERSION"
    
    # Calculate new version based on bump level
    case "{{LEVEL}}" in
        patch)
            NEW_VERSION="$MAJOR.$MINOR.$((PATCH + 1))"
            ;;
        minor)
            NEW_VERSION="$MAJOR.$((MINOR + 1)).0"
            ;;
        major)
            NEW_VERSION="$((MAJOR + 1)).0.0"
            ;;
        *)
            echo "❌ Error: Invalid bump level '{{LEVEL}}'. Use patch, minor, or major."
            exit 1
            ;;
    esac
    
    echo "🚀 New version: $NEW_VERSION"
    
    # Update Cargo.toml
    echo "📝 Updating Cargo.toml..."
    sed -i "s/^version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" Cargo.toml
    
    # Update CHANGELOG.md - move unreleased to new version section
    echo "📝 Updating CHANGELOG.md..."
    TODAY=$(date +%Y-%m-%d)
    
    # Create a temporary file with the new changelog content
    {
        # Keep everything before [Unreleased]
        sed -n '1,/## \[Unreleased\]/p' CHANGELOG.md | head -n -1
        
        # Add new unreleased section
        echo "## [Unreleased]"
        echo ""
        
        # Add new version section with previous unreleased content
        echo "## [$NEW_VERSION] - $TODAY"
        
        # Extract content between [Unreleased] and next version section (excluding the [Unreleased] line itself)
        sed -n '/## \[Unreleased\]/,/## \[[0-9]/p' CHANGELOG.md | sed '1d;$d'
        
        # Add rest of changelog (from first numbered version section onwards)
        sed -n '/## \[[0-9]/,$p' CHANGELOG.md
        
    } > CHANGELOG.md.tmp
    
    mv CHANGELOG.md.tmp CHANGELOG.md
    
    # Run tests to make sure everything still works
    echo "🧪 Running tests..."
    just check
    
    # Commit changes
    echo "📝 Committing changes..."
    git add Cargo.toml Cargo.lock CHANGELOG.md
    git commit -m "chore: bump version to $NEW_VERSION"
    
    # Create and push tag
    echo "🏷️  Creating git tag v$NEW_VERSION..."
    git tag -a "v$NEW_VERSION" -m "Release version $NEW_VERSION"
    
    echo "✅ Version bumped successfully!"
    echo "📋 Next steps:"
    echo "   1. Review the changes: git show HEAD"
    echo "   2. Push changes: git push origin $(git branch --show-current)"
    echo "   3. Push tag: git push origin v$NEW_VERSION"
    echo "   4. Create a GitHub release if desired"
