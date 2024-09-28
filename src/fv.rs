use crate::{get_f64, get_u32, get_when, ParaMap, Result, WhenType};
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

// pub type FVMap = std::collections::HashMap<String, ParaType>;

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

    /// Instantiate a `FutureValue` instance from a hash map with keys of (`rate`, `nper`, `pmt`, `pv` and `when`) in said order
    /// Since [`HashMap`] requires values of same type, we need to wrap into a variant of enum
    pub fn from_map(map: ParaMap) -> Result<Self> {
        let rate = get_f64(&map, "rate")?;
        let nper = get_u32(&map, "nper")?;
        let pmt = get_f64(&map, "pmt")?;
        // let pv = get_f64(&map, "pv").unwrap();
        let pv = get_f64(&map, "pv")?;
        let when = get_when(&map, "when")?;
        Ok(FutureValue {
            rate,
            nper,
            pmt,
            pv,
            when,
        })
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
mod tests {
    use crate::*;

    #[test]
    fn test_fv_from_tuple() {
        let fv = FutureValue::from_tuple((0.075, 20, -2000.0, 0.0, WhenType::End));
        let cond = (fv.rate == 0.075)
            && (fv.nper == 20)
            && (fv.pmt == -2000.0)
            && (fv.pv == 0.0)
            && (fv.when == WhenType::End);

        assert!(cond);
    }

    #[test]
    fn test_fv_from_map() -> Result<()> {
        let mut map = ParaMap::new();
        map.insert("Rate".into(), ParaType::F64(0.075));
        map.insert("nper".into(), ParaType::U32(20));
        map.insert("pmt".into(), ParaType::F64(-2000.0));
        map.insert("pv".into(), ParaType::F64(0.0));
        map.insert("when".into(), ParaType::When(WhenType::End));
        let fv = FutureValue::from_map(map)?;
        let cond = (fv.rate == 0.075)
            && (fv.nper == 20)
            && (fv.pmt == -2000.0)
            && (fv.pv == 0.0)
            && (fv.when == WhenType::End);

        assert!(cond);
        Ok(())
    }

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
