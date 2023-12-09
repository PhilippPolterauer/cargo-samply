# Cargo Samply

a simple integration binary that automates the process of running cargo build with a certain profile and samply afterwards.

## installation

for now you can install it via

```bash
cargo install --git https://github.com/PhilippPolterauer/cargo-samply.git
```

## Example Useage

```bash
cargo new mybinary
cd mybinary
cargo samply
```

when opening the server adress (127.0.0.1:3001) the output should look like the following.
![Samply Web View](doc/samply-web.png)
