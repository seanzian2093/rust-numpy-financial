use crate::{get_f64, get_when, Error, ParaMap, Result, WhenType};
/// # Compute the number of periodic payments

/// ## Parameters
/// * `rate` : an interest rate compounded once per period
/// * `pmt` : payment in each period
/// * `pv` : a present value
/// * `fv` : a future value
/// * `when` : when payments are due [`WhenType`]. Defaults to `When::End`
///
/// ## Return:
/// * `nper`: the number of periodic payments or `None`
///
/// ## Example
/// ```rust
/// use rfinancial::*;
/// let nper = NumberPeriod::from_tuple((0.075, -2000.0, 0.0, 100000.0, WhenType::End));
/// println!("{:#?}'s nper is {:?}", nper, nper.get());
/// ```
///
#[derive(Debug)]
pub struct NumberPeriod {
    rate: f64,
    pmt: f64,
    pv: f64,
    fv: f64,
    when: WhenType,
}

impl NumberPeriod {
    /// Instantiate a `NumberPeriod` instance from a tuple of (`rate`, `pmt`, `pv`, `fv`, and `when`) in said order
    pub fn from_tuple(tup: (f64, f64, f64, f64, WhenType)) -> Self {
        NumberPeriod {
            rate: tup.0,
            pmt: tup.1,
            pv: tup.2,
            fv: tup.3,
            when: tup.4,
        }
    }

    /// Instantiate a `NumberPeriod ` instance from a hash map with keys of (`rate`, `pmt`, `pv`, `fv` and `when`) in said order
    /// Since [`HashMap`] requires values of same type, we need to wrap into a variant of enum
    pub fn from_map(map: ParaMap) -> Result<Self> {
        let op = |err: Error| {
            Error::OtherError(format!(
                "Failed construct an instance of `NumberPeriod` from: `{:?}` <- {}",
                map, err
            ))
        };
        let rate = get_f64(&map, "rate").map_err(|err| op(err))?;
        let pmt = get_f64(&map, "pmt").map_err(|err| op(err))?;
        let pv = get_f64(&map, "pv").map_err(|err| op(err))?;
        let fv = get_f64(&map, "fv").map_err(|err| op(err))?;
        let when = get_when(&map, "when").map_err(|err| op(err))?;
        Ok(NumberPeriod {
            rate,
            pmt,
            pv,
            fv,
            when,
        })
    }

    fn nper(&self) -> Result<Option<f64>> {
        /*
        Solve below equation if rate is not 0
        fv + pv*(1+rate)**nper + pmt*(1+rate*when)/rate*((1+rate)**nper-1) = 0
        but if rate is 0 then
        fv + pv + pmt*nper = 0
        */
        if (self.rate == 0.0) & (self.pmt == 0.0) {
            return Ok(Some(f64::INFINITY));
        }
        if self.rate == 0.0 {
            // We know that pmt_ != 0, we don't need to check for division by 0
            return Ok(Some(-(self.fv + self.pv) / self.pmt));
        }

        if self.rate <= -1.0 {
            return Ok(None);
        }

        // We know that rate_ != 0, we don't need to check for division by 0
        // z = pmt_ * (1.0 + rate_ * when_) / rate_
        // return log((-fv_ + z) / (pv_ + z)) / log(1.0 + rate_)
        let when_f64 = self.when.clone() as u8 as f64;
        let z = self.pmt * (1.0 + self.rate * when_f64) / self.rate;
        Ok(Some(
            ((-self.fv + z) / (self.pv + z)).ln() / (1.0 + self.rate).ln(),
        ))
    }

    /// Get the number of periodic payments from an instance of `NumberPeriod`
    pub fn get(&self) -> Result<Option<f64>> {
        self.nper()
    }
}

#[allow(unused_imports)]
#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_nper_from_tuple() {
        let nper = NumberPeriod::from_tuple((0.075, -2000.0, 0.0, 100000.0, WhenType::End));
        let res = nper.get().unwrap().unwrap();
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
    fn test_nper_from_map() {
        let mut map = ParaMap::new();
        map.insert("rate".into(), ParaType::F64(0.075));
        map.insert("pmt".into(), ParaType::F64(-2000.0));
        map.insert("pv".into(), ParaType::F64(0.0));
        map.insert("fv".into(), ParaType::F64(100000.0));
        map.insert("when".into(), ParaType::When(WhenType::End));

        let nper = NumberPeriod::from_map(map).unwrap();

        let res = nper.get().unwrap().unwrap();
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
    fn test_nper_zero_rate_nonzero_pmt() {
        let nper = NumberPeriod::from_tuple((0.0, -2000.0, 0.0, 100000.0, WhenType::End));
        let res = nper.get().unwrap().unwrap();
        let tgt = 50.0;

        assert!(
            float_close(res, tgt, RTOL, ATOL),
            "{:#?} v.s. {:#?}",
            res,
            tgt
        );
    }

    #[test]
    fn test_nper_zero_rate_zero_pmt() {
        let nper = NumberPeriod::from_tuple((0.0, 0.0, 0.0, 100000.0, WhenType::End));
        let res = nper.get().unwrap().unwrap();
        let tgt = f64::INFINITY;

        assert_eq!(res, tgt, "{:#?} v.s. {:#?}", res, tgt);
    }

    #[test]
    fn test_nper_lt_negative_one_rate() {
        let nper = NumberPeriod::from_tuple((-10.0, 0.0, 0.0, 100000.0, WhenType::End));
        let res = nper.get().unwrap();
        let tgt = None;

        assert_eq!(res, tgt, "{:#?} v.s. {:#?}", res, tgt);
    }

    #[test]
    fn test_nper_err() {
        let mut map = ParaMap::new();
        map.insert("Rate".into(), ParaType::F64(0.075));
        map.insert("pmt".into(), ParaType::F64(-2000.0));
        map.insert("pv".into(), ParaType::F64(0.0));
        map.insert("fv".into(), ParaType::F64(100000.0));
        map.insert("when".into(), ParaType::When(WhenType::End));

        let nper = NumberPeriod::from_map(map);

        let cond = nper.is_err();
        assert!(cond);
    }

    #[test]
    #[ignore = "need to figure out how to produce nan"]
    fn test_nper_nan() {
        let mut map = ParaMap::new();
        map.insert("rate".into(), ParaType::F64(f64::MAX));
        map.insert("pmt".into(), ParaType::F64(-2000.0));
        map.insert("pv".into(), ParaType::F64(0.0));
        map.insert("fv".into(), ParaType::F64(100000.0));
        map.insert("when".into(), ParaType::When(WhenType::End));

        let nper = NumberPeriod::from_map(map).unwrap();

        // need to figure out how to produce nan
        let cond = nper.get().unwrap().unwrap().is_nan();
        assert!(cond);
    }
}
