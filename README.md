## Status

Currently in production, check back here soon to see updates!

![Logo](https://i.imgur.com/X0G455X.jpeg)

Rat is a bare bones, general purpose, statically-typed, multi-paradigm, 'rataturing-complete', personal-project programming language inspired by the Pixar film, Ratatouille. It combines features of Kotlin, Go, and Rust.

## Features

Rat contains four function types:
### fn
```
fn foo(x: int): int {
    ret x + 1
}
```

This is the default function in rat. Return values using the keyword ret.

### fn_
```
fn_ main() {
    io.println("bonjour le monde!")
    rev
}
```

This is a rat-tail function, meaning no explicit return value. Since code inside fn_ functions typically interoperates with stateful behavior, avoid passing in mutable refernces defined outside the function. Use the keyword rev to exit from rat-tails.

### fn?
```
fn? bar() file? {
    let f: file? = file.open("nonexistent_file.txt")
    ret? f
}

```

When a function can return an error, use fn?. Use the keyword ret? to return either a value or the resulting error. 

### fn\
```
fn baz(): int {
    let y: int = 0
    let quux: fn\ = [](z: int): int { 
        ret z + 1 
    }

    ret quux(y)
}

```

For anonymous functions, use fn\. Anonymous functions use either ret, rev, or ret? depending on the return value.
