# Running a `.rat` Program

## 1. Install Rust

Example:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## 2. Write a rat program

Use file extension '.rat'. Various rat example programs are written in the `/data` directory.

## 3. Compile the executable

From the project root, run:

```bash
cargo run -- file=path_to_your_file verbose welcome
```
With additional arguments 'verbose' for debug output and 'welcome' to say hello to our mascot Ratty! All arguments are optional, the default filepath is `/data/compile.rat`
