//! # rfinancial
//! `rfinancial` is a financial crate mimicking `numpy_financial` in Python.
//!
//! ## Initial working version

//! * fv - future value
//! * pmt - payment against loan principal plus interest
//! * nper - number of periodic payments
//! * ipmt - interest portion of a payment

//! ## To be Added

//! * ppmt - payment against loan principle
//! * pv - present value
//! * rate - rate of interest per period
//! * irr - internal rate of return
//! * npv - net present value of a cash flow series
//! * mirr - modified internal rate of return

//! ## Example
//! ```rust
//! use rfinancial::*;
//! let fv = FutureValue::from_tuple((0.075, 20, -2000.0, 0.0, WhenType::End));
//! println!("{:#?}'s fv is {}", fv, fv.get());
//! ```
//!
//! ## Future Work
//! * Add more functions
//! * Add more test cases
//!
//! ## Contribution
//! * Use the crate and feedback
//! * Pull request though the repository

mod fv;
mod nper;
mod pmt;
mod util;

pub use crate::fv::FutureValue;
pub use crate::nper::NumberPeriod;
pub use crate::pmt::Payment;
pub use crate::util::WhenType;
