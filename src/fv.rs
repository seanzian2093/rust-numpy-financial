use crate::WhenType;
/// # Compute the future value

/// ## Parameters
/// * `rate` : an interest rate compounded once per period
/// * `nper` : number of compounding periods
/// * `pmt` : payment in each period
/// * `pv` : present value
/// * `when` : when payments are due [`WhenType`]. Defaults to `When::End`
///
/// ## Return:
/// * `fv`: the value at the end of the `nper` periods, which is used in other modules as parameter
///
/// ## Example
/// ```rust
/// use rfinancial::*;
/// let fv = FutureValue::from_tuple((0.075, 20, -2000.0, 0.0, WhenType::End));
/// println!("{:#?}'s fv is {}", fv, fv.get());
/// ```
///
#[derive(Debug)]
pub struct FutureValue {
    rate: f64,
    nper: u32,
    pmt: f64,
    pv: f64,
    when: WhenType,
}

impl FutureValue {
    /// Instantiate a `FutureValue` instance from a tuple of (`rate`, `nper`, `pmt`, `pv` and `when`) in said order
    pub fn from_tuple(tup: (f64, u32, f64, f64, WhenType)) -> Self {
        FutureValue {
            rate: tup.0,
            nper: tup.1,
            pmt: tup.2,
            pv: tup.3,
            when: tup.4,
        }
    }

    fn fv(&self) -> f64 {
        /*
        Solve below equation if rate is not 0
        fv + pv*(1+rate)**nper + pmt*(1+rate*when)/rate*((1+rate)**nper-1) = 0
        but if rate is 0 then
        fv + pv + pmt*nper = 0
        */
        if self.rate != 0.0 {
            let tmp = (1.0 + self.rate).powf(self.nper as f64);
            let pv_future = self.pv * tmp;
            let when_f64 = self.when.clone() as u8 as f64;
            let pmt_future = self.pmt * (1.0 + self.rate * when_f64) / self.rate * (tmp - 1.0);

            -pv_future - pmt_future
        } else {
            -self.pv - self.pmt * self.nper as f64
        }
    }

    /// Get the future value from an instance of `FutureValue`
    pub fn get(&self) -> f64 {
        self.fv()
    }
}

#[allow(unused_imports)]
#[cfg(test)]
mod test {
    use crate::*;

    #[test]
    fn test_fv_with_begin() {
        let rate = 0.075;
        let nper = 20;
        let pmt = -2000.0;
        let pv = 0.0;
        let when = WhenType::Begin;

        let fv = FutureValue {
            rate,
            nper,
            pmt,
            pv,
            when,
        };
        // npf.fv(0.075, 20, -2000, 0, 1),
        // 93105.064874
        let res = fv.get();
        let tgt = 93105.064874;
        assert!(
            float_close(res, tgt, RTOL, ATOL),
            "{:#?} v.s. {:#?}",
            res,
            tgt
        );
    }

    #[test]
    fn test_fv_with_end() {
        let rate = 0.075;
        let nper = 20;
        let pmt = -2000.0;
        let pv = 0.0;
        let when = WhenType::End;

        let fv = FutureValue {
            rate,
            nper,
            pmt,
            pv,
            when,
        };
        // npf.fv(0.075, 20, -2000, 0, 0),
        // 86609.362673042924,
        let res = fv.get();
        let tgt = 86609.362673042924;
        assert!(
            float_close(res, tgt, RTOL, ATOL),
            "{:#?} v.s. {:#?}",
            res,
            tgt
        );
    }

    #[test]
    fn test_fv_zero_rate() {
        let rate = 0.0;
        let nper = 20;
        let pmt = -100.0;
        let pv = 0.0;
        let when = WhenType::End;

        let fv = FutureValue {
            rate,
            nper,
            pmt,
            pv,
            when,
        };
        let res = fv.get();
        let tgt = 2000.0;
        assert!(
            float_close(res, tgt, RTOL, ATOL),
            "{:#?} v.s. {:#?}",
            res,
            tgt
        );
    }
}
