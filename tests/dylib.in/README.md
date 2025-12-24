# Dynamic Linking Test Project

This project tests cargo-samply's ability to handle dynamically linked binaries.

## How it works

1. `.cargo/config.toml` forces dynamic linking with `rustflags = ["-C", "prefer-dynamic"]`
2. This applies to all build profiles, including the `samply` profile
3. The resulting binary requires `libstd-*.so` at runtime

## Verification

You can verify the binary is dynamically linked:

```bash
# Build with samply profile
cargo build --profile samply

# Check dynamic linking (should show libstd.so not found)
ldd target/samply/dylib-test | grep libstd

# Try to run without library path (should fail)
./target/samply/dylib-test

# Run with cargo-samply (should succeed - our fix)
cargo-samply samply --no-samply

# Run with disabled sysroot injection (should fail - demonstrates the issue)
CARGO_SAMPLY_NO_SYSROOT_INJECTION=1 cargo-samply samply --no-samply
```

## Expected behavior

- Without LD_LIBRARY_PATH: Binary fails with "error while loading shared libraries: libstd-*.so"
- With cargo-samply (fix enabled): Binary runs successfully
- With CARGO_SAMPLY_NO_SYSROOT_INJECTION=1: Binary fails (reproduces issue #17)
