# rfinancial

A financial crate mimicking `numpy_financial` in Python.

## Latest Updates

* Added Logging
  * For example

```rust
fn main() {
    // Initialize logger (using env_logger as the backend)
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .init();

    let rate = Rate::from_tuple((10, 0.0, -3500.0, 10000.0, WhenType::End, 0.1, 1e-6, 100)).expect("Error creating Rate");
    
    match rate.get() {
        Ok(Some(r)) => println!("Result: {:?}", r),
        Ok(None) => println!("No convergence"),
        Err(e) => println!("Error: {:?}", e),
    }
}
```

* Fixed bugs
* Updated API
  * Breaking changes. Check examples and documentation.

## Next Updates

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
* npv - net present value of a cash flow series
* mirr - modified internal rate of return

### To Be Added

* amortization

## Planned Improvements

* Edge cases testing
* Default arguments
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
let fv = FutureValue::from_tuple((0.075, 30, -2000.0, 0.0, WhenType::End)).expect("Error creating FutureValue");
println!("\n{:#?}'s fv is {:?}", fv, fv.get());

// pmt
let pmt = Payment::from_tuple((0.08 / 12.0, 60, 15000.0, 0.0, WhenType::End)).expect("Error creating Payment");
println!("\n{:#?}'s pmt is {:?}", pmt, pmt.get());

// nper
let nper = NumberPeriod::from_tuple((0.075, -2000.0, 0.0, 100000.0, WhenType::End)).expect("Error creating NumberPeriod");
println!("\n{:#?}'s nper is {:?}", nper, nper.get());

// ipmt
let ipmt = InterestPayment::from_tuple((0.1 / 12.0, 1, 24, 2000.0, 0.0, WhenType::End)).expect("Error creating InterestPayment");
println!("\n{:#?}'s ipmt is {:?}", ipmt, ipmt.get());

// ppmt
let ppmt = PrincipalPayment::from_tuple((0.1 / 12.0, 1, 24, 2000.0, 0.0, WhenType::End)).expect("Error creating PrincipalPayment");
println!("\n{:#?}'s ppmt is {:?}", ppmt, ppmt.get());

// pv
let pv = PresentValue::from_tuple((0.075, 20, -2000.0, 0.0, WhenType::End)).expect("Error creating PresentValue");
println!("\n{:#?}'s pv is {:?}", pv, pv.get());

// rate
let rate = Rate::from_tuple((10, 0.0, -3500.0, 10000.0, WhenType::End, 0.1, 1e-6, 100)).expect("Error creating Rate");
println!("\n{:#?}'s rate is {:?}", rate, rate.get());

// irr
let values: Vec<f64> = vec![-150000.0, 15000.0, 25000.0, 35000.0, 45000.0, 60000.0];
let irr = InternalRateReturn::from_vec(values).expect("Error creating IRR");
println!("\n{:#?}'s irr is {:?}", irr, irr.get());

// npv
let tup = (vec![-15000.0, 1500.0, 2500.0, 3500.0, 4500.0, 6000.0], 0.05);
let npv = NetPresentValue::from_tuple(tup).expect("Error creating NPV");
println!("\n{:#?}'s npv is {:?}", npv, npv.get());

// mirr
let tup = (vec![100.0, 200.0, -50.0, 300.00, -200.0], 0.05, 0.06);
let mirr = ModifiedIRR::from_tuple(tup).expect("Error creating MIRR");
println!("\n{:#?}'s mirr is {:?}", mirr, mirr.get());
```

## Future Works

* Add more functions
* Add more test cases

## Contributions

* Use the crate and feedback
* Submit pull request or issues though the GitHub repository
