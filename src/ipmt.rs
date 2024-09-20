use crate::{FutureValue, Payment, WhenType};
/// # Compute the interest portion of a payment

/// ## Parameters
/// * `rate` : an interest rate compounded once per period
/// * `per` : the payment period to calculate the interest amount
/// * `nper` : number of compounding periods
/// * `pv` : a present value
/// * `fv` : a future value
/// * `when` : when payments are due [`WhenType`]. Defaults to `When::End`
///
/// ## Return:
/// * `ipmt`: the interest portion in a payment or `None`
///
/// ## Example
/// ```rust
/// use rfinancial::*;
/// let ipmt = InterestPayment::from_tuple((0.1 / 12.0, 1, 24, 2000.0, 0.0, WhenType::End));
/// println!("{:#?}'s ipmt is {:?}", ipmt, ipmt.get());
/// ```

#[derive(Debug)]
pub struct InterestPayment {
    rate: f64,
    per: u32,
    nper: u32,
    pv: f64,
    fv: f64,
    when: WhenType,
}

impl InterestPayment {
    /// Instantiate a `InterestPayment` instance from a tuple of (`rate`, `per`, `nper`, `pv`, `fv` and `when`) in said order
    pub fn from_tuple(tup: (f64, u32, u32, f64, f64, WhenType)) -> Self {
        InterestPayment {
            rate: tup.0,
            per: tup.1,
            nper: tup.2,
            pv: tup.3,
            fv: tup.4,
            when: tup.5,
        }
    }

    fn ipmt(&self) -> Option<f64> {
        /*
            The total payment is made up of payment against principal plus interest.
            pmt = ppmt + ipmt
        */

        // total payment
        let total_pmt =
            Payment::from_tuple((self.rate, self.nper, self.pv, self.fv, self.when.clone())).get();
        // remaining balance
        // only consider per > 1, i.e. starting from 1st payment
        if self.per >= 1 {
            let rbl = FutureValue::from_tuple((
                self.rate,
                self.per - 1,
                total_pmt,
                self.pv,
                self.when.clone(),
            ))
            .get();

            match self.when {
                WhenType::Begin => {
                    if self.per == 1 {
                        // if payment is made at begin of a period, interest portion is 0 for 1st payment
                        Some(0.0)
                    } else {
                        // discount for 2nd payment and beyond
                        Some(rbl / (1.0 + self.rate) * self.rate)
                    }
                }
                WhenType::End => Some(rbl * self.rate),
            }
            // if 0th or negative-th(not possible though since u32) payments are requested, return None
        } else {
            None
        }
    }

    /// Get the interet payment from an instance of `InterestPayment`
    pub fn get(&self) -> Option<f64> {
        self.ipmt()
    }
}

#[allow(unused_imports)]
#[cfg(test)]
mod test {
    use crate::*;

    #[test]
    fn test_ipmt_with_end() {
        let rate = 0.1 / 12.0;
        let per = 1;
        let nper = 24;
        let pv = 2000.0;
        let fv = 0.0;
        let when = WhenType::End;

        let ipmt = InterestPayment {
            rate,
            per,
            nper,
            pv,
            fv,
            when,
        };
        // npf.ipmt(0.1 / 12, 1, 24, 2000),
        // -16.666667
        let res = ipmt.get().unwrap();
        let tgt = -16.666667;
        assert!(
            float_close(res, tgt, RTOL, ATOL),
            "{:#?} v.s. {:#?}",
            res,
            tgt
        );
    }

    #[test]
    fn test_ipmt_with_begin_1() {
        let rate = 0.0824 / 12.0;
        let per = 1;
        let nper = 12;
        let pv = 2500.0;
        let fv = 0.0;
        let when = WhenType::Begin;

        let ipmt = InterestPayment {
            rate,
            per,
            nper,
            pv,
            fv,
            when,
        };
        // npf.ipmt(0.0824 / 12, 1, 12, 2500, 0, 'begin')
        // array(0.)
        let res = ipmt.get().unwrap();
        let tgt = 0.0;
        assert!(
            float_close(res, tgt, RTOL, ATOL),
            "{:#?} v.s. {:#?}",
            res,
            tgt
        );
    }

    #[test]
    fn test_ipmt_with_begin_2() {
        let rate = 0.0824 / 12.0;
        let per = 2;
        let nper = 12;
        let pv = 2500.0;
        let fv = 0.0;
        let when = WhenType::Begin;

        let ipmt = InterestPayment {
            rate,
            per,
            nper,
            pv,
            fv,
            when,
        };
        // npf.ipmt(0.0824 / 12, 2, 12, 2500, 0, 'begin')
        // array(-15.68165675)
        let res = ipmt.get().unwrap();
        let tgt = -15.68165675;
        assert!(
            float_close(res, tgt, RTOL, ATOL),
            "{:#?} v.s. {:#?}",
            res,
            tgt
        );
    }
    #[test]
    fn test_ipmt_zero_per() {
        let rate = 0.1 / 12.0;
        let per = 0;
        let nper = 24;
        let pv = 2000.0;
        let fv = 0.0;
        let when = WhenType::End;

        let ipmt = InterestPayment {
            rate,
            per,
            nper,
            pv,
            fv,
            when,
        };
        let res = ipmt.get();
        let tgt = None;
        assert_eq!(res, tgt, "{:#?} v.s. {:#?}", res, tgt);
    }
}
