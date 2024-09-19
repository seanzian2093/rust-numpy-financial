# rfinancial

A financial crate mimicking `numpy_financial` in Python.

## Modules

### Initial Working Version

* fv - future value
* pmt - payment against loan principal plus interest
* nper - number of periodic payments
* ipmt - interest portion of a payment
* ppmt - payment against loan principal
* pv - present value
* rate - rate of interest per period
* irr - internal rate of return

### To Be Added

* npv - net present value of a cash flow series
* mirr - modified internal rate of return

## Planned Improvements

* Fix `unused` warnings
* Default arguments
* Error handling
* Logging
* Performance benchmarking
* Function-based solution
  * now is struct-based solution for my other project

## Tests

* All or almost test cases are tested against `numpy_financial`'s result with some exceptions
* `numpy_financial` has some its own issues

## Examples

```rust
use rfinancial::*;

// fv
let fv = FutureValue::from_tuple((0.075, 30, -2000.0, 0.0, WhenType::End));
println!("\n{:#?}'s fv is {}", fv, fv.get());

// pmt
let pmt = Payment::from_tuple((0.08 / 12.0, 60, 15000.0, 0.0, WhenType::End));
println!("\n{:#?}'s pmt is {}", pmt, pmt.get());

// nper
let nper = NumberPeriod::from_tuple((0.075, -2000.0, 0.0, 100000.0, WhenType::End));
println!("\n{:#?}'s nper is {:?}", nper, nper.get());

// ipmt
let ipmt = InterestPayment::from_tuple((0.1 / 12.0, 1, 24, 2000.0, 0.0, WhenType::End));
println!("\n{:#?}'s ipmt is {:?}", ipmt, ipmt.get());

// ppmt
let ppmt = PrincipalPayment::from_tuple((0.1 / 12.0, 1, 24, 2000.0, 0.0, WhenType::End));
println!("\n{:#?}'s ppmt is {:?}", ppmt, ppmt.get());

// pv
let pv = PresentValue::from_tuple((0.075, 20, -2000.0, 0.0, WhenType::End));
println!("\n{:#?}'s pv is {}", pv, pv.get());

// rate
let rate = Rate::from_tuple((10, 0.0, -3500.0, 10000.0, WhenType::End, 0.1, 1e-6, 100));
println!("\n{:#?}'s rate is {:#?}", rate, rate.get());
```

## Future Works

* Add more functions
* Add more test cases

## Contributions

* Use the crate and feedback
* Submit pull request or issues though the GitHub repository
