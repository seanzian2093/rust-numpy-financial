//! # rfinancial
//! `rfinancial` is a financial crate mimicking `numpy_financial` in Python.
//!
//! ## Initial Working Version

//! * fv - future value
//! * pmt - payment against loan principal plus interest
//! * nper - number of periodic payments
//! * ipmt - interest portion of a payment
//! * ppmt - payment against loan principal
//! * pv - present value
//! * rate - rate of interest per period
//! * irr - internal rate of return

//! ## To Be Added

//! * npv - net present value of a cash flow series
//! * mirr - modified internal rate of return

//! ## Tests
//! * All test cases are tested against `numpy_financial`'s result with some exceptions
//! * `numpy_financial` has some its own issues

//! ## Example
//! You will find example in each module page
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
//! * Submit pull request or issues though the GitHub repository

mod fv;
mod ipmt;
mod irr;
mod nper;
mod npv;
mod pmt;
mod ppmt;
mod pv;
mod rate;
mod util;

pub use crate::fv::FutureValue;
pub use crate::ipmt::InterestPayment;
pub use crate::irr::InternalRateReturn;
pub use crate::nper::NumberPeriod;
pub use crate::npv::NetPresentValue;
pub use crate::pmt::Payment;
pub use crate::ppmt::PrincipalPayment;
pub use crate::pv::PresentValue;
pub use crate::rate::Rate;
pub use crate::util::*;
