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
* [example_ipmt.rs](example_ipmt.rs) - interest portion of a payment (`ipmt`),
  showing both the function-based (`ipmt(...)`) and struct-based
  (`InterestPayment::from_tuple(...)`) APIs.
* [example_pmt.rs](example_pmt.rs) - payment against loan principal plus
  interest (`pmt`), showing both the function-based (`pmt(...)`) and
  struct-based (`Payment::from_tuple(...)`) APIs.
* [example_irr.rs](example_irr.rs) - internal rate of return (`irr`), showing
  both the function-based (`irr(...)`) and struct-based
  (`InternalRateReturn::from_vec(...)`) APIs.
* [example_mirr.rs](example_mirr.rs) - modified internal rate of return
  (`mirr`), showing both the function-based (`mirr(...)`) and struct-based
  (`ModifiedIRR::from_tuple(...)`) APIs.
* [example_nper.rs](example_nper.rs) - number of periodic payments (`nper`),
  showing both the function-based (`nper(...)`) and struct-based
  (`NumberPeriod::from_tuple(...)`) APIs.
* [example_npv.rs](example_npv.rs) - net present value (`npv`), showing both
  the function-based (`npv(...)`) and struct-based
  (`NetPresentValue::from_tuple(...)`) APIs.
* [example_ppmt.rs](example_ppmt.rs) - payment against loan principal
  (`ppmt`), showing both the function-based (`ppmt(...)`) and struct-based
  (`PrincipalPayment::from_tuple(...)`) APIs.
* [example_rate.rs](example_rate.rs) - interest rate (`rate`), showing both
  the function-based (`rate(...)`) and struct-based (`Rate::from_tuple(...)`)
  APIs.
