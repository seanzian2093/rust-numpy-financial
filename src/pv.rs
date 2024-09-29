use crate::{get_f64, get_u32, get_when, util::WhenType, Error, ParaMap, Result};
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
/// ```rust
/// use rfinancial::*;
/// let pv = PresentValue::from_tuple((0.075, 20, -2000.0, 0.0, WhenType::End));
/// println!("{:#?}'s pv is {:?}", pv, pv.get());
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

    /// Instantiate a `PresentValue` instance from a hash map with keys of (`rate`, `nper`,`pmt`, `fv`, and `when`) in said order
    /// Since [`HashMap`] requires values of same type, we need to wrap into a variant of enum
    pub fn from_map(map: ParaMap) -> Result<Self> {
        let op = |err: Error| {
            Error::OtherError(format!(
                "Failed construct an instance of `PresentValue` from: `{:?}` <- {}",
                map, err
            ))
        };

        let rate = get_f64(&map, "rate").map_err(|err| op(err))?;
        let nper = get_u32(&map, "nper").map_err(|err| op(err))?;
        let pmt = get_f64(&map, "pmt").map_err(|err| op(err))?;
        let fv = get_f64(&map, "fv").map_err(|err| op(err))?;
        let when = get_when(&map, "when").map_err(|err| op(err))?;
        Ok(PresentValue {
            rate,
            nper,
            pmt,
            fv,
            when,
        })
    }

    fn fv(&self) -> Result<f64> {
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
            Ok(-(self.fv + self.pmt * fact) / temp)
        } else {
            Ok(-self.fv - self.pmt * self.nper as f64)
        }
    }

    /// Get the future value from an instance of `PresentValue`
    pub fn get(&self) -> Result<f64> {
        self.fv()
    }
}

#[allow(unused_imports)]
#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_pv_from_tuple() {
        let pv = PresentValue::from_tuple((0.07, 20, 12000.0, 0.0, WhenType::End));

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
