# Yet Another Really Rusty Lox

This software is not meant to be useful. It's merely for practice and learning.

## But what is this?

This is a WIP implemetation of the _Lox_ language described in the delightful
light novel [_Crafting Interpreters_](http://craftinginterpreters.com/). The
lecture is highly recommended - it's a delectable romantic thriller, full of ups
and down, relatable characters, and touching on universal human suffering with
refreshing lightness.

For now, there's only the tree-walk interpreter.

Yes, the "really" wasn't _really_ needed in the name. Ye better not complain
lest ye be thrown ta th' sharks.

## Running _yarrlox_

First of all, [get Rust if you need to.](https://rustup.rs/)

Run the REPL:

```
cargo run
```

Run a _Lox_ program:

```
cargo run -- scripts/fib.lox
```

Install it into your `PATH` if you like it a lot:

```
cargo install --path .
```

## Differences from the Book

- Binary operation precedence/associativity is handled with a precedence
  climbing algorithm.
- The original jlox implementation seems to cleverly exploit Java's memory
  management and garbage collection so that the reader doesn't have to think
  about those... yet. For my tree-walk interpreter, I decided to pass small
  values by value and put bigger ones (functions, strings) in ref-counted
  pointers. This probably has implications about cycles and all that jazz. It's
  probably not ideal without deeper thought, but it lets me get on with the book
  without spending too much time here.
- In Rust, there's no built-in mechanism for making every instance of a type
  uniquely identifiable. That's why the parser assigns a unique ID to every
  variable reference. Since those IDs are contiguous, our `locals` data
  collected by the resolver doesn't need to be a hashmap. It's a `Vec` instead.
- No visitor pattern in the tree-walk interpreter. I couldn't find enough
  justification for doing that and opted for enums and pattern-matching instead.

## License

Dual licensed under MIT and Apache 2.0 at your option, like most Rust project.
