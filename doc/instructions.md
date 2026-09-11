# Running a `.rat` Program

## 1. Set filepath

In `src/main.rs`, find this line:

```rust
let filepath = String::from("data/compile.rat");
```

Replace `"data/compile.rat"` with the path to the `.rat` program you want to run.

## 2. Choose compilation mode

On the subsequent line there is a function call `compile(filepath, ...)`. The second argument is a debug flag:

- **`compile(filepath, true)`** runs the full compilation pipeline with debug printing. 
- **`compile(filepath, false)`** only generates the executable.

```rust
compile(filepath, true);  // verbose: execute and print pipeline stages
// or
compile(filepath, false); // quiet: just execute pipeline 
```

## 3. Build and run

From the project root, run:

```bash
cargo run
```
