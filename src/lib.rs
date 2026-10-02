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
//! * npv - net present value of a cash flow series
//! * mirr - modified internal rate of return

//! ## To Be Added
//! * amortization

//! ## Tests
//! * All test cases are tested against `numpy_financial`'s result with some exceptions
//! * `numpy_financial` has some its own issues

//! ## Example
//! You will find example in each module page
//! ```rust
//! use rfinancial::*;
//! let fv = FutureValue::from_tuple((0.075, 20, -2000.0, 0.0, WhenType::End)).expect("Error creating FutureValue");
//! println!("{:#?}'s fv is {:?}", fv, fv.get());
//! ```
//!
//! ## Future Work
//! * Add more functions
//! * Add more test cases
//!
//! ## Contribution
//! * Use the crate and feedback
//! * Submit pull request or issues though the GitHub repository

mod error;
mod fv;
mod ipmt;
mod irr;
mod mirr;
mod nper;
mod npv;
mod pmt;
mod ppmt;
mod pv;
mod rate;
mod util;

pub use crate::error::{Error, Result};
pub use crate::fv::{FutureValue, fv, fv_from_map};
pub use crate::ipmt::{InterestPayment, ipmt, ipmt_from_map};
pub use crate::irr::{InternalRateReturn, irr, irr_from_map};
pub use crate::mirr::{ModifiedIRR, mirr, mirr_from_map};
pub use crate::nper::{NumberPeriod, nper, nper_from_map};
pub use crate::npv::{NetPresentValue, npv, npv_from_map};
pub use crate::pmt::{Payment, pmt, pmt_from_map};
pub use crate::ppmt::{PrincipalPayment, ppmt, ppmt_from_map};
pub use crate::pv::{PresentValue, pv, pv_from_map};
pub use crate::rate::{Rate, rate, rate_from_map};
pub use crate::util::*;
