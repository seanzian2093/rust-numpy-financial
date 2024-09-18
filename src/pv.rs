use crate::util::{float_close, WhenType, ATOL, RTOL};
/// # Compute the present value.

/// ## Parameters
/// * `rate` : an interest rate compounded once per period
/// * `nper` : number of compounding periods
/// * `pmt` : payment in each period
/// * `fv` : future value
/// * `when` : when payments are due [`WhenType`]. Defaults to `When::End`
///
/// ## Return:
/// * `pv`: the present value of a series of payments, which is used in other modules as parameter
///
/// ## Example
/// ```rust
/// use rfinancial::*;
/// let pv = PresentValue::from_tuple((0.075, 20, -2000.0, 0.0, WhenType::End));
/// println!("{:#?}'s pv is {}", pv, pv.get());
/// ```
#[derive(Debug)]
pub struct PresentValue {
    rate: f64,
    nper: u32,
    pmt: f64,
    fv: f64,
    when: WhenType,
}

impl PresentValue {
    /// Instantiate a `PresentValue` instance from a tuple of (`rate`, `nper`, `pmt`, `fv` and `when`) in said order
    pub fn from_tuple(tup: (f64, u32, f64, f64, WhenType)) -> Self {
        PresentValue {
            rate: tup.0,
            nper: tup.1,
            pmt: tup.2,
            fv: tup.3,
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
            let temp = (1.0 + self.rate).powf(self.nper as f64);
            let when_f64 = self.when.clone() as u8 as f64;
            let fact = (1.0 + self.rate * when_f64) * (temp - 1.0) / self.rate;
            -(self.fv + self.pmt * fact) / temp
        } else {
            -self.fv - self.pmt * self.nper as f64
        }
    }

    /// Get the future value from an instance of `PresentValue`
    pub fn get(&self) -> f64 {
        self.fv()
    }
}

mod test {
    use super::*;

    #[test]
    fn test_pv_with_begin() {
        let rate = 0.07;
        let nper = 20;
        let pmt = 12000.0;
        let fv = 0.0;
        let when = WhenType::Begin;

        let pv = PresentValue {
            rate,
            nper,
            pmt,
            fv,
            when,
        };
        // npf.pv(0.07, 20, 12000, 0, 'begin')
        // -136027.14291242755
        let res = pv.get();
        let tgt = -136027.14291242755;
        assert!(
            float_close(res, tgt, RTOL, ATOL),
            "{:#?} v.s. {:#?}",
            res,
            tgt
        );
    }

    #[test]
    fn test_pv_with_end() {
        let rate = 0.07;
        let nper = 20;
        let pmt = 12000.0;
        let fv = 0.0;
        let when = WhenType::End;

        let pv = PresentValue {
            rate,
            nper,
            pmt,
            fv,
            when,
        };
        // npf.pv(0.07, 20, 12000, 0)
        // -127128.17
        let res = pv.get();
        let tgt = -127128.17094619398;
        assert!(
            float_close(res, tgt, RTOL, ATOL),
            "{:#?} v.s. {:#?}",
            res,
            tgt
        );
    }

    #[test]
    fn test_pv_zero_rate() {
        let rate = 0.0;
        let nper = 20;
        let pmt = 12000.0;
        let fv = 0.0;
        let when = WhenType::End;

        let pv = PresentValue {
            rate,
            nper,
            pmt,
            fv,
            when,
        };
        // npf.pv(0.07, 20, 12000, 0)
        // -240000.0
        let res = pv.get();
        let tgt = -240000.0;
        assert!(
            float_close(res, tgt, RTOL, ATOL),
            "{:#?} v.s. {:#?}",
            res,
            tgt
        );
    }
}
