mod fv;
mod nper;
mod pmt;
mod util;

use crate::fv::FutureValue;
use crate::nper::NumberPeriod;
use crate::pmt::Payment;
use crate::util::WhenType;
fn main() {
    // fv
    let fv = FutureValue::from_tuple((0.075, 20, -2000.0, 0.0, WhenType::End));
    println!("{:#?}'s fv is {}", fv, fv.get());

    // pmt
    let pmt = Payment::from_tuple((0.08 / 12.0, 60, 15000.0, 0.0, WhenType::End));
    println!("{:#?}'s pmt is {}", pmt, pmt.get());
}
