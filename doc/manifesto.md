# Rat_

## Features

fn -> function
 - second class
 - pure

 ```
fn foo(x: int): int {
    ret x + 1
}
 ```

 fn/ -> lambda function
 ```
fn foo(x: int): int {
    fn/ inc = [](x: int): int {
        x + 1
    }

    ret inc(x); 
}
 ```

fn? -> optional function
 - first class
 - pure
```
fn? foo(x: op int): op int {
    match x {
        x => x + 1
        Null => Null
    }
}
```

fn_ -> void function
 - first class
 - stateful
 - pure
```
fn_ printer(x: int) {
    println(x)
    rev
}

fn foo(x: int): int {
    printer(x)

    ret x + 1
}

```


