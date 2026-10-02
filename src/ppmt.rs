use crate::{
    Error, FromMap, FromTuple, ParaMap, Result, WhenType, get_f64, get_u32, get_when, ipmt, pmt,
};
/// # Compute the payment against loan principal
/// ## Parameters
/// * `rate` : an interest rate compounded once per period
/// * `per` : the payment period to calculate the interest amount
/// * `nper` : number of compounding periods
/// * `pv` : a present value
/// * `fv` : a future value
/// * `when` : when payments are due [`WhenType`]. Defaults to `When::End`
///
/// ## Return:
/// * `ppmt`: the payment against loan principal
///
/// ## Example
/// Struct-based:
/// ```rust
/// use rfinancial::*;
/// let result = PrincipalPayment::from_tuple((0.1 / 12.0, 1, 24, 2000.0, 0.0, WhenType::End)).expect("Error creating PrincipalPayment");
/// println!("{:#?}'s ppmt is {:?}", result, result.get());
/// ```
/// Function-based:
/// ```rust
/// use rfinancial::*;
/// let result = ppmt(0.1 / 12.0, 1, 24, 2000.0, 0.0, WhenType::End).expect("Error computing ppmt");
/// println!("ppmt is {:?}", result);
/// ```
#[derive(Debug)]
pub struct PrincipalPayment {
    rate: f64,
    per: u32,
    nper: u32,
    pv: f64,
    fv: f64,
    when: WhenType,
}

impl PrincipalPayment {
    /// Instantiate a `PrincipalPayment` instance from a tuple of (`rate`, `per`, `nper`, `pv`, `fv` and `when`) in said order
    pub fn from_tuple(tup: (f64, u32, u32, f64, f64, WhenType)) -> Result<Self> {
        Ok(PrincipalPayment {
            rate: tup.0,
            per: tup.1,
            nper: tup.2,
            pv: tup.3,
            fv: tup.4,
            when: tup.5,
        })
    }

    /// Instantiate a `PrincipalPayment` instance from a hash map with keys of (`rate`, `per`, `nper`,`pv`, `fv`, and `when`) in said order
    /// Since [`HashMap`] requires values of same type, we need to wrap into a variant of enum
    pub fn from_map(map: ParaMap) -> Result<Self> {
        let op = |err: Error| {
            Error::OtherError(format!(
                "Failed construct an instance of `PrincipalPayment` from: `{:?}` <- {}",
                map, err
            ))
        };

        let rate = get_f64(&map, "rate").map_err(&op)?;
        let per = get_u32(&map, "per").map_err(&op)?;
        let nper = get_u32(&map, "nper").map_err(&op)?;
        let pv = get_f64(&map, "pv").map_err(&op)?;
        let fv = get_f64(&map, "fv").map_err(&op)?;
        let when = get_when(&map, "when").map_err(op)?;
        Ok(PrincipalPayment {
            rate,
            per,
            nper,
            pv,
            fv,
            when,
        })
    }

    fn ppmt(&self) -> Result<Option<f64>> {
        ppmt(self.rate, self.per, self.nper, self.pv, self.fv, self.when)
    }

    /// Get the interet payment from an instance of `PrincipalPayment`
    pub fn get(&self) -> Result<Option<f64>> {
        self.ppmt()
    }
}
impl FromTuple<(f64, u32, u32, f64, f64, WhenType)> for PrincipalPayment {
    fn from_tuple(tup: (f64, u32, u32, f64, f64, WhenType)) -> Result<Self> {
        PrincipalPayment::from_tuple(tup)
    }
}

impl FromMap for PrincipalPayment {
    fn from_map(map: ParaMap) -> Result<Self> {
        PrincipalPayment::from_map(map)
    }
}

/// Compute the payment against loan principal
/// ## Parameters
/// * `rate` : an interest rate compounded once per period
/// * `per` : the payment period to calculate the interest amount
/// * `nper` : number of compounding periods
/// * `pv` : a present value
/// * `fv` : a future value
/// * `when` : when payments are due [`WhenType`]
///
/// ## Return:
/// * the payment against loan principal or `None`
///
/// ## Example
/// ```rust
/// use rfinancial::*;
/// let result = ppmt(0.1 / 12.0, 1, 24, 2000.0, 0.0, WhenType::End).expect("Error computing ppmt");
/// println!("ppmt is {:?}", result);
/// ```
pub fn ppmt(
    rate: f64,
    per: u32,
    nper: u32,
    pv: f64,
    fv: f64,
    when: WhenType,
) -> Result<Option<f64>> {
    /*
        The total payment is made up of payment against principal plus interest.
        pmt = ppmt + ipmt
    */

    // total payment
    let total_pmt = pmt(rate, nper, pv, fv, when)?;
    // interest payment
    let interest = ipmt(rate, per, nper, pv, fv, when)?;

    let ppmt = interest.map(|value| total_pmt - value);

    Ok(ppmt)
}

/// Compute the payment against loan principal from a hash map with keys of
/// (`rate`, `per`, `nper`, `pv`, `fv`, and `when`)
/// Since [`HashMap`] requires values of same type, we need to wrap into a variant of enum
/// ## Example
/// ```rust
/// use rfinancial::*;
/// let mut map = ParaMap::new();
/// map.insert("rate".into(), ParaType::F64(0.1 / 12.0));
/// map.insert("per".into(), ParaType::U32(1));
/// map.insert("nper".into(), ParaType::U32(24));
/// map.insert("pv".into(), ParaType::F64(2000.0));
/// map.insert("fv".into(), ParaType::F64(0.0));
/// map.insert("when".into(), ParaType::When(WhenType::End));
/// let result = ppmt_from_map(map).expect("Error computing ppmt");
/// println!("ppmt is {:?}", result);
/// ```
pub fn ppmt_from_map(map: ParaMap) -> Result<Option<f64>> {
    let op = |err: Error| {
        Error::OtherError(format!(
            "Failed to compute `ppmt` from: `{:?}` <- {}",
            map, err
        ))
    };

    let rate = get_f64(&map, "rate").map_err(&op)?;
    let per = get_u32(&map, "per").map_err(&op)?;
    let nper = get_u32(&map, "nper").map_err(&op)?;
    let pv = get_f64(&map, "pv").map_err(&op)?;
    let fv = get_f64(&map, "fv").map_err(&op)?;
    let when = get_when(&map, "when").map_err(op)?;
    ppmt(rate, per, nper, pv, fv, when)
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_ppmt_from_tuple() {
        let ppmt =
            PrincipalPayment::from_tuple((0.1 / 12.0, 1, 60, 55000.0, 0.0, WhenType::End)).unwrap();
        // npf.ppmt(0.1 / 12, 1, 60, 55000)
        // -710.254125786425
        let res = ppmt.get().unwrap().unwrap();
        let tgt = -710.254125786425;
        assert!(
            float_close(res, tgt, RTOL, ATOL),
            "{:#?} v.s. {:#?}",
            res,
            tgt
        );
    }

    #[test]
    fn test_ppmt_from_map() {
        let mut map = ParaMap::new();
        map.insert("rate".into(), ParaType::F64(0.1 / 12.0));
        map.insert("per".into(), ParaType::U32(1));
        map.insert("nper".into(), ParaType::U32(60));
        map.insert("pv".into(), ParaType::F64(55000.0));
        map.insert("fv".into(), ParaType::F64(0.0));
        map.insert("when".into(), ParaType::When(WhenType::End));
        let ppmt = PrincipalPayment::from_map(map).unwrap();
        // npf.ppmt(0.1 / 12, 1, 60, 55000)
        // -710.254125786425
        let res = ppmt.get().unwrap().unwrap();
        let tgt = -710.254125786425;
        assert!(
            float_close(res, tgt, RTOL, ATOL),
            "{:#?} v.s. {:#?}",
            res,
            tgt
        );
    }

    #[test]
    fn test_ppmt_with_end() {
        let rate = 0.1 / 12.0;
        let per = 1;
        let nper = 60;
        let pv = 55000.0;
        let fv = 0.0;
        let when = WhenType::End;

        let ppmt = PrincipalPayment {
            rate,
            per,
            nper,
            pv,
            fv,
            when,
        };
        // npf.ppmt(0.1 / 12, 1, 60, 55000)
        // -710.254125786425
        let res = ppmt.get().unwrap().unwrap();
        let tgt = -710.254125786425;
        assert!(
            float_close(res, tgt, RTOL, ATOL),
            "{:#?} v.s. {:#?}",
            res,
            tgt
        );
    }

    #[test]
    fn test_ppmt_with_begin() {
        let rate = 0.1 / 12.0;
        let per = 1;
        let nper = 60;
        let pv = 55000.0;
        let fv = 0.0;
        let when = WhenType::Begin;

        let ppmt = PrincipalPayment {
            rate,
            per,
            nper,
            pv,
            fv,
            when,
        };
        // npf.ppmt(0.1 / 12, 1, 60, 55000, 0, 'begin')
        // -1158.9297115237273
        let res = ppmt.get().unwrap().unwrap();
        let tgt = -1158.9297115237273;
        assert!(
            float_close(res, tgt, RTOL, ATOL),
            "{:#?} v.s. {:#?}",
            res,
            tgt
        );
    }

    #[test]
    fn test_ppmt_zero_per() {
        let rate = 0.1 / 12.0;
        let per = 0;
        let nper = 24;
        let pv = 2000.0;
        let fv = 0.0;
        let when = WhenType::End;

        let ppmt = PrincipalPayment {
            rate,
            per,
            nper,
            pv,
            fv,
            when,
        };
        let res = ppmt.get().unwrap();
        let tgt = None;
        assert_eq!(res, tgt, "{:#?} v.s. {:#?}", res, tgt);
    }

    #[test]
    fn test_ppmt_err() {
        let mut map = ParaMap::new();
        map.insert("Rate".into(), ParaType::F64(0.1 / 12.0));
        map.insert("per".into(), ParaType::U32(1));
        map.insert("nper".into(), ParaType::U32(60));
        map.insert("pv".into(), ParaType::F64(55000.0));
        map.insert("fv".into(), ParaType::F64(0.0));
        map.insert("when".into(), ParaType::When(WhenType::End));
        let ppmt = PrincipalPayment::from_map(map);
        let cond = ppmt.is_err();
        assert!(cond);
    }

    #[test]
    fn test_ppmt_function() {
        // npf.ppmt(0.1 / 12, 1, 60, 55000)
        // -710.254125786425
        let res = super::ppmt(0.1 / 12.0, 1, 60, 55000.0, 0.0, WhenType::End)
            .unwrap()
            .unwrap();
        let tgt = -710.254125786425;
        assert!(
            float_close(res, tgt, RTOL, ATOL),
            "{:#?} v.s. {:#?}",
            res,
            tgt
        );
    }

    #[test]
    fn test_ppmt_from_map_function() {
        let mut map = ParaMap::new();
        map.insert("rate".into(), ParaType::F64(0.1 / 12.0));
        map.insert("per".into(), ParaType::U32(1));
        map.insert("nper".into(), ParaType::U32(60));
        map.insert("pv".into(), ParaType::F64(55000.0));
        map.insert("fv".into(), ParaType::F64(0.0));
        map.insert("when".into(), ParaType::When(WhenType::End));

        // npf.ppmt(0.1 / 12, 1, 60, 55000)
        // -710.254125786425
        let res = super::ppmt_from_map(map).unwrap().unwrap();
        let tgt = -710.254125786425;
        assert!(
            float_close(res, tgt, RTOL, ATOL),
            "{:#?} v.s. {:#?}",
            res,
            tgt
        );
    }
}
