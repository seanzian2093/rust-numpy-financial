use crate::{get_f64, get_u32, get_when, InterestPayment, ParaMap, Payment, Result, WhenType};
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
/// ```rust
/// use rfinancial::*;
/// let ppmt = PrincipalPayment::from_tuple((0.1 / 12.0, 1, 24, 2000.0, 0.0, WhenType::End));
/// println!("{:#?}'s ppmt is {:?}", ppmt, ppmt.get());
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
    pub fn from_tuple(tup: (f64, u32, u32, f64, f64, WhenType)) -> Self {
        PrincipalPayment {
            rate: tup.0,
            per: tup.1,
            nper: tup.2,
            pv: tup.3,
            fv: tup.4,
            when: tup.5,
        }
    }

    /// Instantiate a `PrincipalPayment` instance from a hash map with keys of (`rate`, `per`, `nper`,`pv`, `fv`, and `when`) in said order
    /// Since [`HashMap`] requires values of same type, we need to wrap into a variant of enum
    pub fn from_map(map: ParaMap) -> Self {
        let rate = get_f64(&map, "rate").unwrap();
        let per = get_u32(&map, "per").unwrap();
        let nper = get_u32(&map, "nper").unwrap();
        let pv = get_f64(&map, "pv").unwrap();
        let fv = get_f64(&map, "fv").unwrap();
        let when = get_when(&map, "when").unwrap();
        PrincipalPayment {
            rate,
            per,
            nper,
            pv,
            fv,
            when,
        }
    }

    fn ppmt(&self) -> Result<Option<f64>> {
        /*
            The total payment is made up of payment against principal plus interest.
            pmt = ppmt + ipmt
        */

        // total payment
        let total_pmt =
            Payment::from_tuple((self.rate, self.nper, self.pv, self.fv, self.when.clone())).get();
        // interest payment
        let ipmt = InterestPayment::from_tuple((
            self.rate,
            self.per,
            self.nper,
            self.pv,
            self.fv,
            self.when.clone(),
        ))
        .get()?;

        let ppmt = match ipmt {
            Some(value) => Some(total_pmt - value),
            None => None,
        };

        Ok(ppmt)
    }

    /// Get the interet payment from an instance of `PrincipalPayment`
    pub fn get(&self) -> Result<Option<f64>> {
        self.ppmt()
    }
}

#[allow(unused_imports)]
#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_ppmt_from_tuple() {
        let ppmt = PrincipalPayment::from_tuple((0.1 / 12.0, 1, 60, 55000.0, 0.0, WhenType::End));
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
        let ppmt = PrincipalPayment::from_map(map);
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
}
