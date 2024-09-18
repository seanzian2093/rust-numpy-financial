# rfinancial

A financial crate mimicking `numpy_financial` in Python.

## Module

### Initial Working Version

* fv - future value
* pmt - payment against loan principal plus interest
* nper - number of periodic payments
* ipmt - interest portion of a payment
* ppmt - payment against loan principal
* pv - present value
* rate - rate of interest per period

### To Be Added

* irr - internal rate of return
* npv - net present value of a cash flow series
* mirr - modified internal rate of return

## Improvements

* Default arguments
* Error handling
* Logging
* Performance benchmarking

## Example

```rust
use rfinancial::*;
let fv = FutureValue::from_tuple((0.075, 20, -2000.0, 0.0, WhenType::End));
println!("{:#?}'s fv is {}", fv, fv.get());
```
