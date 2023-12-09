# Cargo Samply

[![Continuous integration](https://github.com/PhilippPolterauer/cargo-samply/actions/workflows/ci.yml/badge.svg)](https://github.com/PhilippPolterauer/cargo-samply/actions/workflows/ci.yml)

a simple integration binary that automates the process of running `cargo build` with a certain profile and `samply` afterwards.
It installs [samply](https://github.com/mstange/samply) if it is not available using `cargo install`.

## Installation

for now you can install it from crates.io or directly from github.com

```bash
# crates.io
cargo install cargo-samply
# or from git
cargo install --git https://github.com/PhilippPolterauer/cargo-samply.git
```

## Example Usage

The usage is quite simple

```console
$ cargo new mybinary
$ cd mybinary
$ cargo samply
cargo.toml: ~/rust/mybinary/Cargo.toml
'samply' profile was added to 'Cargo.toml'
   Compiling mybinary v0.1.0 (~/rust/mybinary)
    Finished samply [optimized + debuginfo] target(s) in 0.18s
Hello, world!
Local server listening at http://127.0.0.1:3001
```

when opening the server address (127.0.0.1:3001) the output should look like the following.
![Samply Web View](https://raw.githubusercontent.com/PhilippPolterauer/cargo-samply/main/doc/samply-web.png)

## Advanced Usecases with custom arguments

there is a test package at `test/testpackage/` which can be used for highlighting the following options.

`cargo-samply` respects the default-run argument:

```console
$ cargo samply
running 'cargo locate-project'
cargo.toml: /home/philipp/rust/cargo-samply/tests/testpackage/Cargo.toml
running 'cargo build --profile samply'
running 'samply record target/samply/main'
```

`cargo-samply` can be targeted at certain binaries e.g. `--example`

```console
$ cargo samply --example hello
running 'cargo locate-project'
cargo.toml: /home/philipp/rust/cargo-samply/tests/testpackage/Cargo.toml
running 'cargo build --profile samply --example hello'
running 'samply record target/samply/examples/hello'
```

or `--bin`

```console
$ cargo samply --bin another
running 'cargo locate-project'
cargo.toml: /home/philipp/rust/cargo-samply/tests/testpackage/Cargo.toml
running 'cargo build --profile samply --bin another'
running 'samply record target/samply/another'
```

additional arguments can be passed after `--`

```console
$ cargo samply --bin another -- 'new text'

```