use crate::util::WhenType;
/// # Compute the payment against loan principal plus interest

/// ## Parameters
/// * `rate` : an interest rate compounded once per period
/// * `nper` : number of periodic payments
/// * `pv` : a present value
/// * `fv` : a future value
/// * `when` : when payments are due [`WhenType`]. Defaults to `When::End`
///
/// ## Return:
/// * `pmt`: payment in each period
///
/// ## Example
/// ```rust
/// use rfinancial::*;
/// let pmt = Payment::from_tuple((0.08 / 12.0, 60, 15000.0, 0.0, WhenType::End));
/// println!("{:#?}'s pmt is {}", pmt, pmt.get());
/// ```
#[derive(Debug)]
pub struct Payment {
    rate: f64,
    nper: u32,
    pv: f64,
    fv: f64,
    when: WhenType,
}

impl Payment {
    /// Instantiate a `Payment` instance from a tuple of (`rate`, `nper`, `pv`, `fv` and `when`) in said order
    pub fn from_tuple(tup: (f64, u32, f64, f64, WhenType)) -> Self {
        Payment {
            rate: tup.0,
            nper: tup.1,
            pv: tup.2,
            fv: tup.3,
            when: tup.4,
        }
    }

    fn pmt(&self) -> f64 {
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
            let fact = (1.0 + self.rate * when_f64) / self.rate * (tmp - 1.0);
            -(self.fv + pv_future) / fact
        } else {
            -(self.pv + self.fv) / self.nper as f64
        }
    }
    /// Get the payment from an instance of `Payment`
    pub fn get(&self) -> f64 {
        self.pmt()
    }
}

#[allow(unused_imports)]
#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_pmt_with_end() {
        let rate = 0.08 / 12.0;
        let nper = 60;
        let pv = 15000.0;
        let fv = 0.0;
        let when = WhenType::End;

        let pmt = Payment {
            rate,
            nper,
            pv,
            fv,
            when,
        };
        // res = npf.pmt(0.08 / 12, 5 * 12, 15000)
        // tgt = -304.145914
        let tgt = -304.145914;
        let res = pmt.get();
        assert!(
            float_close(res, tgt, RTOL, ATOL),
            "{:#?} v.s. {:#?}",
            res,
            tgt
        );
    }
    #[test]
    fn test_pmt_zero_rate() {
        let rate = 0.0;
        let nper = 60;
        let pv = 15000.0;
        let fv = 0.0;
        let when = WhenType::End;

        let pmt = Payment {
            rate,
            nper,
            pv,
            fv,
            when,
        };
        // res = npf.pmt(0.0, 5 * 12, 15000)
        // tgt = -250.0
        let tgt = -250.0;
        let res = pmt.get();
        assert!(
            float_close(res, tgt, RTOL, ATOL),
            "{:#?} v.s. {:#?}",
            res,
            tgt
        );
    }
}
