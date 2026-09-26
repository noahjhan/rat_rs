# Running a `.rat` Program

## 1. Install Rust

Example:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## 2. Write a rat program

Use file extension `.rat`. Various rat example programs are written in the `/data` directory.

```
fn_ main() {
    io::println("bonjour le monde!")
    rev
}
```

## 3. Compile the executable

From the project root, run:

```bash
cargo run -- file=path_to_your_file verbose bonjour 
```

All arguments are optional; the default filepath is `/data/compile.rat`, with additional arguments `verbose` for debug output and `bonjour` to meet our mascot Ratty! 
