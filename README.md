# rfinancial

A Rust financial library that mimics `numpy_financial` in Python, providing functions for financial calculations including future value, present value, and internal rate of return, etc, and more to come.

## Features

### Multiple API Styles

- **Struct-based API**: Type-safe, composable, with clear semantics
- **Function-based API**: Direct computation without creating intermediate structs
- **Builder Pattern**: Fluent, chainable API with sensible defaults (for complex functions)

### Comprehensive Financial Functions

- Future Value (FV)
- Present Value (PV)
- Payment calculations (PMT)
- Number of periods (NPER)
- Interest portion of payment (IPMT)
- Principal portion of payment (PPMT)
- Interest rate calculation (RATE)
- Internal Rate of Return (IRR)
- Modified Internal Rate of Return (MIRR)
- Net Present Value (NPV)

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
rfinancial = "1.3"
```

### Struct-based API (Recommended for complex use cases)

```rust
use rfinancial::*;

// Future Value - struct-based
let result = FutureValue::from_tuple((0.075, 30, -2000.0, 0.0, WhenType::End))?;
println!("FV: {}", result.get()?);
```

### Function-based API (Direct computation)

```rust
use rfinancial::*;

// Future Value - function-based (simpler, no intermediate struct)
let result = fv(0.075, 30, -2000.0, 0.0, WhenType::End)?;
println!("FV: {}", result);
```

### Builder Pattern API (Best for complex parameters)

The builder pattern is available for functions with many parameters, providing a fluent, chainable API with sensible defaults:

```rust
use rfinancial::*;

// Rate calculation with builder - much more readable!
let result = RateBuilder::new()
    .nper(10)
    .pv(-3500.0)
    .fv(10000.0)
    .guess(0.1)         // Initial guess
    .tol(1e-6)          // Convergence tolerance
    .maxiter(100)       // Max iterations
    .build()?;

match result.get()? {
    Some(r) => println!("Interest rate: {:.2}%", r * 100.0),
    None => println!("No convergence"),
}

// Interest Payment with defaults
let result = InterestPaymentBuilder::new()
    .rate(0.1 / 12.0)
    .per(1)
    .nper(24)
    .pv(2000.0)
    .build()?;  // Uses default: when=WhenType::End, fv=0.0

println!("Interest portion: {:?}", result.get()?);
```

## Complete Examples

See `examples`

## Backward Compatibility

All original APIs remain functional:

- `Struct::from_tuple()` - Constructor from tuples
- `Struct::from_map()` - Constructor from HashMap
- `function_name()` - Direct function calls
- Builder pattern is purely additive

## Logging

Enable debug logging with `env_logger`:

```rust
fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .init();

    let rate = RateBuilder::new()
        .nper(10)
        .pv(-3500.0)
        .fv(10000.0)
        .build()
        .expect("Error creating Rate");
    
    match rate.get() {
        Ok(Some(r)) => println!("Result: {:?}", r),
        Ok(None) => println!("No convergence"),
        Err(e) => println!("Error: {:?}", e),
    }
}
```

## Modules

### Core Financial Functions

- **fv** - Future value
- **pv** - Present value
- **pmt** - Payment against loan principal plus interest
- **nper** - Number of periodic payments
- **rate** - Interest rate per period
- **ipmt** - Interest portion of a payment
- **ppmt** - Principal portion of a payment
- **irr** - Internal rate of return
- **mirr** - Modified internal rate of return
- **npv** - Net present value of cash flows

## Planned Additions

- Amortization schedule generation
- Default parameter support for optional fields

## Contributing

We welcome contributions! To get involved:

1. Use the crate and provide feedback
2. Report issues or suggest improvements
3. Submit pull requests via GitHub

## License

Licensed under the MIT License - see LICENSE file for details.
