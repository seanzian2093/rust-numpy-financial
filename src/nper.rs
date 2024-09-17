use std::f32::consts::LN_10;

use crate::util::{float_close, WhenType, ATOL, RTOL};
#[derive(Debug)]
pub struct NumberPeriod {
    rate: f64,
    pmt: f64,
    pv: f64,
    fv: f64,
    when: WhenType,
}

impl NumberPeriod {
    fn from_tuple(tup: (f64, f64, f64, f64, WhenType)) -> Self {
        NumberPeriod {
            rate: tup.0,
            pmt: tup.1,
            pv: tup.2,
            fv: tup.3,
            when: tup.4,
        }
    }

    fn nper(&self) -> Option<f64> {
        /*
        Solve below equation if rate is not 0
        fv + pv*(1+rate)**nper + pmt*(1+rate*when)/rate*((1+rate)**nper-1) = 0
        but if rate is 0 then
        fv + pv + pmt*nper = 0
        */
        if (self.rate == 0.0) & (self.pmt == 0.0) {
            return Some(f64::INFINITY);
        }
        if self.rate == 0.0 {
            // We know that pmt_ != 0, we don't need to check for division by 0
            return Some(-(self.fv + self.pv) / self.pmt);
        }

        if self.rate <= -1.0 {
            return None;
        }

        // We know that rate_ != 0, we don't need to check for division by 0
        // z = pmt_ * (1.0 + rate_ * when_) / rate_
        // return log((-fv_ + z) / (pv_ + z)) / log(1.0 + rate_)
        let when_f64 = self.when.clone() as u8 as f64;
        let z = self.pmt * (1.0 + self.rate * when_f64) / self.rate;
        Some(((-self.fv + z) / (self.pv + z)).ln() / (1.0 + self.rate).ln())
    }

    pub fn get(&self) -> Option<f64> {
        self.nper()
    }
}

mod test {

    use std::f64::INFINITY;

    use super::*;

    #[test]
    fn test_simple_case() {
        let nper = NumberPeriod::from_tuple((0.075, -2000.0, 0.0, 100000.0, WhenType::End));
        let res = nper.get().unwrap();
        let tgt = 21.544944;

        assert!(
            float_close(res, tgt, RTOL, ATOL),
            "{:#?} v.s. {:#?}",
            res,
            tgt
        );
        // npf.nper([0, 0.075], -2000, 0, 100000),
        // [50, 21.544944],  # Computed using Google Sheet's NPER
    }

    #[test]
    fn test_zero_rate_nonzero_pmt() {
        let nper = NumberPeriod::from_tuple((0.0, -2000.0, 0.0, 100000.0, WhenType::End));
        let res = nper.get().unwrap();
        let tgt = 50.0;

        assert!(
            float_close(res, tgt, RTOL, ATOL),
            "{:#?} v.s. {:#?}",
            res,
            tgt
        );
    }

    #[test]
    fn test_zero_rate_zero_pmt() {
        let nper = NumberPeriod::from_tuple((0.0, 0.0, 0.0, 100000.0, WhenType::End));
        let res = nper.get().unwrap();
        let tgt = f64::INFINITY;

        assert_eq!(res, tgt, "{:#?} v.s. {:#?}", res, tgt);
    }

    #[test]
    fn test_lt_negative_one_rate() {
        let nper = NumberPeriod::from_tuple((-10.0, 0.0, 0.0, 100000.0, WhenType::End));
        let res = nper.get();
        let tgt = None;

        assert_eq!(res, tgt, "{:#?} v.s. {:#?}", res, tgt);
    }
}
