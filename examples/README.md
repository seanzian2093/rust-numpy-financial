# Examples

Each `.rs` file in this folder is an independent, runnable example. Cargo
auto-discovers any file here that contains a `fn main()` — no `Cargo.toml`
entry is required.

## Running an example

```sh
cargo run --example <filename-without-.rs>
```

For example:

```sh
cargo run --example example_pv
```

## Building/checking all examples

```sh
cargo build --examples
cargo test --examples
```

## Adding a new example

1. Create a new file, e.g. `examples/example_<module>.rs`.
2. Add `use rfinancial::*;` and a `fn main() { ... }` that demonstrates the
   module's API (both the struct-based and function-based styles, where
   available).
3. Run it with `cargo run --example example_<module>` to verify it compiles
   and produces the expected output.

## Current examples

* [example_pv.rs](example_pv.rs) - present value (`pv`), showing both the
  function-based (`pv(...)`) and struct-based (`PresentValue::from_tuple(...)`)
  APIs.
