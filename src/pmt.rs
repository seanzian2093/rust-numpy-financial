use crate::{Error, FromMap, FromTuple, ParaMap, Result, WhenType, get_f64, get_u32, get_when};
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
/// Struct-based:
/// ```rust
/// use rfinancial::*;
/// let pmt = Payment::from_tuple((0.08 / 12.0, 60, 15000.0, 0.0, WhenType::End)).expect("Error creating Payment");
/// println!("{:#?}'s pmt is {:?}", pmt, pmt.get());
/// ```
/// Function-based:
/// ```rust
/// use rfinancial::*;
/// let result = pmt(0.08 / 12.0, 60, 15000.0, 0.0, WhenType::End).expect("Error computing pmt");
/// println!("pmt is {:?}", result);
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
    pub fn from_tuple(tup: (f64, u32, f64, f64, WhenType)) -> Result<Self> {
        Ok(Payment {
            rate: tup.0,
            nper: tup.1,
            pv: tup.2,
            fv: tup.3,
            when: tup.4,
        })
    }

    /// Instantiate a `Payment` instance from a hash map with keys of (`rate`, `nper`, `pv`, `fv`, and `when`) in said order
    /// Since [`HashMap`] requires values of same type, we need to wrap into a variant of enum
    pub fn from_map(map: ParaMap) -> Result<Self> {
        let op = |err: Error| {
            Error::OtherError(format!(
                "Failed construct an instance of `Payment` from: `{:?}` <- {}",
                map, err
            ))
        };

        let rate = get_f64(&map, "rate").map_err(&op)?;
        let nper = get_u32(&map, "nper").map_err(&op)?;
        let pv = get_f64(&map, "pv").map_err(&op)?;
        let fv = get_f64(&map, "fv").map_err(&op)?;
        let when = get_when(&map, "when").map_err(op)?;
        Ok(Payment {
            rate,
            nper,
            pv,
            fv,
            when,
        })
    }

    fn pmt(&self) -> Result<f64> {
        pmt(self.rate, self.nper, self.pv, self.fv, self.when)
    }

    /// Get the payment from an instance of `Payment`
    pub fn get(&self) -> Result<f64> {
        self.pmt()
    }
}

impl FromTuple<(f64, u32, f64, f64, WhenType)> for Payment {
    fn from_tuple(tup: (f64, u32, f64, f64, WhenType)) -> Result<Self> {
        Payment::from_tuple(tup)
    }
}

impl FromMap for Payment {
    fn from_map(map: ParaMap) -> Result<Self> {
        Payment::from_map(map)
    }
}

/// Compute the payment against loan principal plus interest
/// ## Parameters
/// * `rate` : an interest rate compounded once per period
/// * `nper` : number of periodic payments
/// * `pv` : a present value
/// * `fv` : a future value
/// * `when` : when payments are due [`WhenType`]
///
/// ## Return:
/// * payment in each period
///
/// ## Example
/// ```rust
/// use rfinancial::*;
/// let result = pmt(0.08 / 12.0, 60, 15000.0, 0.0, WhenType::End).expect("Error computing pmt");
/// println!("pmt is {:?}", result);
/// ```
pub fn pmt(rate: f64, nper: u32, pv: f64, fv: f64, when: WhenType) -> Result<f64> {
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
        let fact = (1.0 + rate * when_f64) / rate * (tmp - 1.0);
        Ok(-(fv + pv_future) / fact)
    } else {
        Ok(-(pv + fv) / nper as f64)
    }
}

/// Compute the payment from a hash map with keys of (`rate`, `nper`, `pv`, `fv`, and `when`)
/// Since [`HashMap`] requires values of same type, we need to wrap into a variant of enum
/// ## Example
/// ```rust
/// use rfinancial::*;
/// let mut map = ParaMap::new();
/// map.insert("rate".into(), ParaType::F64(0.08 / 12.0));
/// map.insert("nper".into(), ParaType::U32(60));
/// map.insert("pv".into(), ParaType::F64(15000.0));
/// map.insert("fv".into(), ParaType::F64(0.0));
/// map.insert("when".into(), ParaType::When(WhenType::End));
/// let result = pmt_from_map(map).expect("Error computing pmt");
/// println!("pmt is {:?}", result);
/// ```
pub fn pmt_from_map(map: ParaMap) -> Result<f64> {
    let op = |err: Error| {
        Error::OtherError(format!(
            "Failed to compute `pmt` from: `{:?}` <- {}",
            map, err
        ))
    };

    let rate = get_f64(&map, "rate").map_err(&op)?;
    let nper = get_u32(&map, "nper").map_err(&op)?;
    let pv = get_f64(&map, "pv").map_err(&op)?;
    let fv = get_f64(&map, "fv").map_err(&op)?;
    let when = get_when(&map, "when").map_err(op)?;
    pmt(rate, nper, pv, fv, when)
}

#[cfg(test)]
mod tests {
    use crate::{ATOL, ParaMap, ParaType, Payment, RTOL, WhenType, float_close};

    #[test]
    fn test_pmt_from_tuple() {
        let pmt = Payment::from_tuple((0.08 / 12.0, 60, 15000.0, 0.0, WhenType::End)).unwrap();

        let res = pmt.get().unwrap();
        let tgt = -304.145914;
        assert!(
            float_close(res, tgt, RTOL, ATOL),
            "{:#?} v.s. {:#?}",
            res,
            tgt
        );
    }

    #[test]
    fn test_pmt_from_map() {
        let mut map = ParaMap::new();
        map.insert("rate".into(), ParaType::F64(0.08 / 12.0));
        map.insert("nper".into(), ParaType::U32(60));
        map.insert("pv".into(), ParaType::F64(15000.0));
        map.insert("fv".into(), ParaType::F64(0.0));
        map.insert("when".into(), ParaType::When(WhenType::End));

        let pmt = Payment::from_map(map).unwrap();

        let res = pmt.get().unwrap();
        let tgt = -304.145914;
        assert!(
            float_close(res, tgt, RTOL, ATOL),
            "{:#?} v.s. {:#?}",
            res,
            tgt
        );
    }

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
        let res = pmt.get().unwrap();
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
        let res = pmt.get().unwrap();
        assert!(
            float_close(res, tgt, RTOL, ATOL),
            "{:#?} v.s. {:#?}",
            res,
            tgt
        );
    }

    #[test]
    fn test_pmt_err() {
        let mut map = ParaMap::new();
        map.insert("Rate".into(), ParaType::F64(0.08 / 12.0));
        map.insert("nper".into(), ParaType::U32(60));
        map.insert("pv".into(), ParaType::F64(15000.0));
        map.insert("fv".into(), ParaType::F64(0.0));
        map.insert("when".into(), ParaType::When(WhenType::End));

        let pmt = Payment::from_map(map);
        let cond = pmt.is_err();
        assert!(cond)
    }

    #[test]
    fn test_pmt_function() {
        // res = npf.pmt(0.08 / 12, 5 * 12, 15000)
        // tgt = -304.145914
        let res = super::pmt(0.08 / 12.0, 60, 15000.0, 0.0, WhenType::End).unwrap();
        let tgt = -304.145914;
        assert!(
            float_close(res, tgt, RTOL, ATOL),
            "{:#?} v.s. {:#?}",
            res,
            tgt
        );
    }

    #[test]
    fn test_pmt_from_map_function() {
        let mut map = ParaMap::new();
        map.insert("rate".into(), ParaType::F64(0.08 / 12.0));
        map.insert("nper".into(), ParaType::U32(60));
        map.insert("pv".into(), ParaType::F64(15000.0));
        map.insert("fv".into(), ParaType::F64(0.0));
        map.insert("when".into(), ParaType::When(WhenType::End));

        // res = npf.pmt(0.08 / 12, 5 * 12, 15000)
        // tgt = -304.145914
        let res = super::pmt_from_map(map).unwrap();
        let tgt = -304.145914;
        assert!(
            float_close(res, tgt, RTOL, ATOL),
            "{:#?} v.s. {:#?}",
            res,
            tgt
        );
    }
}
