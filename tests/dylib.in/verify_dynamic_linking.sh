#!/bin/bash
set -e

echo "=== Verifying Dynamic Linking Test Setup ==="
echo ""

# Build the binary
echo "1. Building with samply profile..."
cargo build --profile samply --quiet

# Check if binary is dynamically linked
echo "2. Checking if binary is dynamically linked..."
if ldd target/samply/dylib-test 2>&1 | grep -q "libstd.*not found"; then
    echo "   ✓ Binary is dynamically linked (libstd.so not found in system)"
else
    echo "   ✗ ERROR: Binary does not appear to be dynamically linked!"
    echo "   Expected to see 'libstd-*.so => not found' in ldd output"
    ldd target/samply/dylib-test
    exit 1
fi

# Try to run without library path (should fail)
echo "3. Testing binary fails without LD_LIBRARY_PATH..."
if ./target/samply/dylib-test 2>&1 | grep -q "error while loading shared libraries"; then
    echo "   ✓ Binary fails as expected (cannot find libstd.so)"
else
    echo "   ✗ ERROR: Binary should have failed but didn't!"
    exit 1
fi

echo ""
echo "=== All checks passed! ===" 
echo "The test project correctly uses dynamic linking."
echo "Run 'cargo-samply samply --no-samply' to test the fix."
