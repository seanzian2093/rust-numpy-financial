use crate::{Error, FromMap, FromTuple, ParaMap, Result, WhenType, get_f64, get_u32, get_when};
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
/// Struct-based:
/// ```rust
/// use rfinancial::*;
/// let result = FutureValue::from_tuple((0.075, 20, -2000.0, 0.0, WhenType::End)).expect("Error creating FutureValue");
/// println!("{:#?}'s fv is {:?}", result, result.get());
/// ```
/// Function-based:
/// ```rust
/// use rfinancial::*;
/// let result = fv(0.075, 20, -2000.0, 0.0, WhenType::End).expect("Error computing fv");
/// println!("fv is {:?}", result);
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
    pub fn from_tuple(tup: (f64, u32, f64, f64, WhenType)) -> Result<Self> {
        Ok(FutureValue {
            rate: tup.0,
            nper: tup.1,
            pmt: tup.2,
            pv: tup.3,
            when: tup.4,
        })
    }

    /// Instantiate a `FutureValue` instance from a hash map with keys of (`rate`, `nper`, `pmt`, `pv` and `when`) in said order.
    /// Since `HashMap` requires values of same type, we need to wrap into a variant of enum
    pub fn from_map(map: ParaMap) -> Result<Self> {
        let op = |err: Error| {
            Error::OtherError(format!(
                "Failed construct an instance of `FutureValue` from: `{:?}` <- {}",
                map, err
            ))
        };

        let rate = get_f64(&map, "rate").map_err(&op)?;
        let nper = get_u32(&map, "nper").map_err(&op)?;
        let pmt = get_f64(&map, "pmt").map_err(&op)?;
        let pv = get_f64(&map, "pv").map_err(&op)?;
        let when = get_when(&map, "when").map_err(op)?;

        Ok(FutureValue {
            rate,
            nper,
            pmt,
            pv,
            when,
        })
    }

    fn fv(&self) -> Result<f64> {
        fv(self.rate, self.nper, self.pmt, self.pv, self.when)
    }

    /// Get the future value from an instance of `FutureValue`
    pub fn get(&self) -> Result<f64> {
        self.fv()
    }
}

impl FromTuple<(f64, u32, f64, f64, WhenType)> for FutureValue {
    fn from_tuple(tup: (f64, u32, f64, f64, WhenType)) -> Result<Self> {
        FutureValue::from_tuple(tup)
    }
}

impl FromMap for FutureValue {
    fn from_map(map: ParaMap) -> Result<Self> {
        FutureValue::from_map(map)
    }
}

/// Compute the future value at the end of `nper` periods
/// ## Parameters
/// * `rate` : an interest rate compounded once per period
/// * `nper` : number of compounding periods
/// * `pmt` : payment in each period
/// * `pv` : present value
/// * `when` : when payments are due [`WhenType`]
///
/// ## Return:
/// * the value at the end of the `nper` periods
///
/// ## Example
/// ```rust
/// use rfinancial::*;
/// let result = fv(0.075, 20, -2000.0, 0.0, WhenType::End).expect("Error computing fv");
/// println!("fv is {:?}", result);
/// ```
pub fn fv(rate: f64, nper: u32, pmt: f64, pv: f64, when: WhenType) -> Result<f64> {
    /*
    Solve below equation if rate is not 0
    fv + pv*(1+rate)**nper + pmt*(1+rate*when)/rate*((1+rate)**nper-1) = 0
    but if rate is 0 then
    fv + pv + pmt*nper = 0
    */
    if rate != 0.0 {
        let tmp = (1.0 + rate).powf(nper as f64);
        let pv_future = pv * tmp;
        let when_f64 = when as u8 as f64;
        let pmt_future = pmt * (1.0 + rate * when_f64) / rate * (tmp - 1.0);

        Ok(-pv_future - pmt_future)
    } else {
        Ok(-pv - pmt * nper as f64)
    }
}

/// Compute the future value from a hash map with keys of (`rate`, `nper`, `pmt`, `pv`, and `when`).
/// Since `HashMap` requires values of same type, we need to wrap into a variant of enum.
/// ## Example
/// ```rust
/// use rfinancial::*;
/// let mut map = ParaMap::new();
/// map.insert("rate".into(), ParaType::F64(0.075));
/// map.insert("nper".into(), ParaType::U32(20));
/// map.insert("pmt".into(), ParaType::F64(-2000.0));
/// map.insert("pv".into(), ParaType::F64(0.0));
/// map.insert("when".into(), ParaType::When(WhenType::End));
/// let result = fv_from_map(map).expect("Error computing fv");
/// println!("fv is {:?}", result);
/// ```
pub fn fv_from_map(map: ParaMap) -> Result<f64> {
    let op = |err: Error| {
        Error::OtherError(format!(
            "Failed to compute `fv` from: `{:?}` <- {}",
            map, err
        ))
    };

    let rate = get_f64(&map, "rate").map_err(&op)?;
    let nper = get_u32(&map, "nper").map_err(&op)?;
    let pmt = get_f64(&map, "pmt").map_err(&op)?;
    let pv = get_f64(&map, "pv").map_err(&op)?;
    let when = get_when(&map, "when").map_err(op)?;
    fv(rate, nper, pmt, pv, when)
}

#[cfg(test)]
mod tests {
    use core::f64;

    use crate::*;

    #[test]
    fn test_fv_from_tuple() {
        let fv = FutureValue::from_tuple((0.075, 20, -2000.0, 0.0, WhenType::End)).unwrap();
        let cond = (fv.rate == 0.075)
            && (fv.nper == 20)
            && (fv.pmt == -2000.0)
            && (fv.pv == 0.0)
            && (fv.when == WhenType::End);

        assert!(cond);
    }

    #[test]
    fn test_fv_from_map() {
        let mut map = ParaMap::new();
        map.insert("rate".into(), ParaType::F64(0.075));
        map.insert("nper".into(), ParaType::U32(20));
        map.insert("pmt".into(), ParaType::F64(-2000.0));
        map.insert("pv".into(), ParaType::F64(0.0));
        map.insert("when".into(), ParaType::When(WhenType::End));
        let fv = FutureValue::from_map(map).unwrap();
        let cond = (fv.rate == 0.075)
            && (fv.nper == 20)
            && (fv.pmt == -2000.0)
            && (fv.pv == 0.0)
            && (fv.when == WhenType::End);

        assert!(cond);
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
        let res = fv.get().unwrap();
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
        let res = fv.get().unwrap();
        let tgt = 86609.36267304292;
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
        let res = fv.get().unwrap();
        let tgt = 2000.0;
        assert!(
            float_close(res, tgt, RTOL, ATOL),
            "{:#?} v.s. {:#?}",
            res,
            tgt
        );
    }

    #[test]
    fn test_fv_function() {
        // npf.fv(0.075, 20, -2000, 0, 0),
        // 86609.362673042924,
        let res = super::fv(0.075, 20, -2000.0, 0.0, WhenType::End).unwrap();
        let tgt = 86609.36267304292;
        assert!(
            float_close(res, tgt, RTOL, ATOL),
            "{:#?} v.s. {:#?}",
            res,
            tgt
        );
    }

    #[test]
    fn test_fv_from_map_function() {
        let mut map = ParaMap::new();
        map.insert("rate".into(), ParaType::F64(0.075));
        map.insert("nper".into(), ParaType::U32(20));
        map.insert("pmt".into(), ParaType::F64(-2000.0));
        map.insert("pv".into(), ParaType::F64(0.0));
        map.insert("when".into(), ParaType::When(WhenType::End));

        // npf.fv(0.075, 20, -2000, 0, 0),
        // 86609.362673042924,
        let res = super::fv_from_map(map).unwrap();
        let tgt = 86609.36267304292;
        assert!(
            float_close(res, tgt, RTOL, ATOL),
            "{:#?} v.s. {:#?}",
            res,
            tgt
        );
    }

    #[test]
    fn test_fv_nan() {
        let mut map = ParaMap::new();
        // map.insert("rate".into(), ParaType::F64(f64::MAX));
        map.insert("rate".into(), ParaType::F64(f64::MIN));
        map.insert("nper".into(), ParaType::U32(100));
        map.insert("pmt".into(), ParaType::F64(-2000.0));
        map.insert("pv".into(), ParaType::F64(0.0));
        map.insert("when".into(), ParaType::When(WhenType::End));
        let fv = FutureValue::from_map(map).unwrap();
        let cond = fv.get().unwrap().is_nan();

        assert!(cond);
    }

    #[test]
    fn test_fv_err() {
        let mut map = ParaMap::new();
        map.insert("Rate".into(), ParaType::F64(0.075));
        map.insert("nper".into(), ParaType::U32(100));
        map.insert("pmt".into(), ParaType::F64(-2000.0));
        map.insert("pv".into(), ParaType::F64(0.0));
        map.insert("when".into(), ParaType::When(WhenType::End));
        let fv = FutureValue::from_map(map);
        let cond = fv.is_err();

        assert!(cond);
    }
}
