use crate::{
    Error, FromMap, FromTuple, ParaMap, Result, get_f64, get_u32, get_when, util::WhenType,
};
/// # Compute the present value
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
/// Struct-based:
/// ```rust
/// use rfinancial::*;
/// let result = PresentValue::from_tuple((0.075, 20, -2000.0, 0.0, WhenType::End)).expect("Error creating PresentValue");
/// println!("{:#?}'s pv is {:?}", result, result.get());
/// ```
/// Function-based:
/// ```rust
/// use rfinancial::*;
/// let result = pv(0.075, 20, -2000.0, 0.0, WhenType::End).expect("Error computing pv");
/// println!("pv is {:?}", result);
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
    pub fn from_tuple(tup: (f64, u32, f64, f64, WhenType)) -> Result<Self> {
        Ok(PresentValue {
            rate: tup.0,
            nper: tup.1,
            pmt: tup.2,
            fv: tup.3,
            when: tup.4,
        })
    }

    /// Instantiate a `PresentValue` instance from a hash map with keys of (`rate`, `nper`,`pmt`, `fv`, and `when`) in said order.
    /// Since `HashMap` requires values of same type, we need to wrap into a variant of enum
    pub fn from_map(map: ParaMap) -> Result<Self> {
        let op = |err: Error| {
            Error::OtherError(format!(
                "Failed construct an instance of `PresentValue` from: `{:?}` <- {}",
                map, err
            ))
        };

        let rate = get_f64(&map, "rate").map_err(&op)?;
        let nper = get_u32(&map, "nper").map_err(&op)?;
        let pmt = get_f64(&map, "pmt").map_err(&op)?;
        let fv = get_f64(&map, "fv").map_err(&op)?;
        let when = get_when(&map, "when").map_err(op)?;
        Ok(PresentValue {
            rate,
            nper,
            pmt,
            fv,
            when,
        })
    }

    fn pv(&self) -> Result<f64> {
        pv(self.rate, self.nper, self.pmt, self.fv, self.when)
    }

    /// Get the present value from an instance of `PresentValue`
    pub fn get(&self) -> Result<f64> {
        self.pv()
    }
}

impl FromTuple<(f64, u32, f64, f64, WhenType)> for PresentValue {
    fn from_tuple(tup: (f64, u32, f64, f64, WhenType)) -> Result<Self> {
        PresentValue::from_tuple(tup)
    }
}

impl FromMap for PresentValue {
    fn from_map(map: ParaMap) -> Result<Self> {
        PresentValue::from_map(map)
    }
}

/// Compute the present value of a series of payments
/// ## Parameters
/// * `rate` : an interest rate compounded once per period
/// * `nper` : number of compounding periods
/// * `pmt` : payment in each period
/// * `fv` : future value
/// * `when` : when payments are due [`WhenType`]
///
/// ## Return:
/// * the present value of a series of payments
///
/// ## Example
/// ```rust
/// use rfinancial::*;
/// let result = pv(0.075, 20, -2000.0, 0.0, WhenType::End).expect("Error computing pv");
/// println!("pv is {:?}", result);
/// ```
pub fn pv(rate: f64, nper: u32, pmt: f64, fv: f64, when: WhenType) -> Result<f64> {
    /*
    Solve below equation if rate is not 0
    fv + pv*(1+rate)**nper + pmt*(1+rate*when)/rate*((1+rate)**nper-1) = 0
    but if rate is 0 then
    fv + pv + pmt*nper = 0
    */
    if rate != 0.0 {
        let temp = (1.0 + rate).powf(nper as f64);
        let when_f64 = when as u8 as f64;
        let fact = (1.0 + rate * when_f64) * (temp - 1.0) / rate;
        Ok(-(fv + pmt * fact) / temp)
    } else {
        Ok(-fv - pmt * nper as f64)
    }
}

/// Compute the present value from a hash map with keys of (`rate`, `nper`, `pmt`, `fv`, and `when`).
/// Since `HashMap` requires values of same type, we need to wrap into a variant of enum
/// ## Example
/// ```rust
/// use rfinancial::*;
/// let mut map = ParaMap::new();
/// map.insert("rate".into(), ParaType::F64(0.075));
/// map.insert("nper".into(), ParaType::U32(20));
/// map.insert("pmt".into(), ParaType::F64(-2000.0));
/// map.insert("fv".into(), ParaType::F64(0.0));
/// map.insert("when".into(), ParaType::When(WhenType::End));
/// let result = pv_from_map(map).expect("Error computing pv");
/// println!("pv is {:?}", result);
/// ```
pub fn pv_from_map(map: ParaMap) -> Result<f64> {
    let op = |err: Error| {
        Error::OtherError(format!(
            "Failed to compute `pv` from: `{:?}` <- {}",
            map, err
        ))
    };

    let rate = get_f64(&map, "rate").map_err(&op)?;
    let nper = get_u32(&map, "nper").map_err(&op)?;
    let pmt = get_f64(&map, "pmt").map_err(&op)?;
    let fv = get_f64(&map, "fv").map_err(&op)?;
    let when = get_when(&map, "when").map_err(op)?;
    pv(rate, nper, pmt, fv, when)
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_pv_from_tuple() {
        let pv = PresentValue::from_tuple((0.07, 20, 12000.0, 0.0, WhenType::End)).unwrap();

        // npf.pv(0.07, 20, 12000, 0)
        // -127128.17
        let res = pv.get().unwrap();
        let tgt = -127128.17094619398;
        assert!(
            float_close(res, tgt, RTOL, ATOL),
            "{:#?} v.s. {:#?}",
            res,
            tgt
        );
    }

    #[test]
    fn test_pv_from_map() {
        let mut map = ParaMap::new();
        map.insert("rate".into(), ParaType::F64(0.07));
        map.insert("nper".into(), ParaType::U32(20));
        map.insert("pmt".into(), ParaType::F64(12000.0));
        map.insert("fv".into(), ParaType::F64(0.0));
        map.insert("when".into(), ParaType::When(WhenType::End));
        let pv = PresentValue::from_map(map).unwrap();

        // npf.pv(0.07, 20, 12000, 0)
        // -127128.17
        let res = pv.get().unwrap();
        let tgt = -127128.17094619398;
        assert!(
            float_close(res, tgt, RTOL, ATOL),
            "{:#?} v.s. {:#?}",
            res,
            tgt
        );
    }

    #[test]
    fn test_pv_function() {
        // npf.pv(0.07, 20, 12000, 0)
        // -127128.17
        let res = super::pv(0.07, 20, 12000.0, 0.0, WhenType::End).unwrap();
        let tgt = -127128.17094619398;
        assert!(
            float_close(res, tgt, RTOL, ATOL),
            "{:#?} v.s. {:#?}",
            res,
            tgt
        );
    }

    #[test]
    fn test_pv_from_map_function() {
        let mut map = ParaMap::new();
        map.insert("rate".into(), ParaType::F64(0.07));
        map.insert("nper".into(), ParaType::U32(20));
        map.insert("pmt".into(), ParaType::F64(12000.0));
        map.insert("fv".into(), ParaType::F64(0.0));
        map.insert("when".into(), ParaType::When(WhenType::End));

        // npf.pv(0.07, 20, 12000, 0)
        // -127128.17
        let res = super::pv_from_map(map).unwrap();
        let tgt = -127128.17094619398;
        assert!(
            float_close(res, tgt, RTOL, ATOL),
            "{:#?} v.s. {:#?}",
            res,
            tgt
        );
    }

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
        let res = pv.get().unwrap();
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
        let res = pv.get().unwrap();
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
        let res = pv.get().unwrap();
        let tgt = -240000.0;
        assert!(
            float_close(res, tgt, RTOL, ATOL),
            "{:#?} v.s. {:#?}",
            res,
            tgt
        );
    }

    #[test]
    fn test_pv_err() {
        let mut map = ParaMap::new();
        map.insert("Rate".into(), ParaType::F64(0.07));
        map.insert("nper".into(), ParaType::U32(20));
        map.insert("pmt".into(), ParaType::F64(12000.0));
        map.insert("fv".into(), ParaType::F64(0.0));
        map.insert("when".into(), ParaType::When(WhenType::End));
        let pv = PresentValue::from_map(map);
        assert!(pv.is_err())
    }
}
