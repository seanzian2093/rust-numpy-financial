use crate::{
    Error, FromMap, FromTuple, ParaMap, Result, WhenType, fv::fv as future_value, get_f64, get_u32,
    get_when, pmt,
};
/// # Compute the interest portion of a payment
/// ## Parameters
/// * `rate` : an interest rate compounded once per period
/// * `per` : the payment period to calculate the interest amount
/// * `nper` : number of compounding periods
/// * `pv` : a present value
/// * `fv` : a future value
/// * `when` : when payments are due [`WhenType`]. Defaults to `When::End`
///
/// ## Return:
/// * `ipmt`: the interest portion in a payment or `None`
///
/// ## Example
/// Struct-based:
/// ```rust
/// use rfinancial::*;
/// let result = InterestPayment::from_tuple((0.1 / 12.0, 1, 24, 2000.0, 0.0, WhenType::End)).expect("Error creating InterestPayment");
/// println!("{:#?}'s ipmt is {:?}", result, result.get());
/// ```
/// Function-based:
/// ```rust
/// use rfinancial::*;
/// let result = ipmt(0.1 / 12.0, 1, 24, 2000.0, 0.0, WhenType::End).expect("Error computing ipmt");
/// println!("ipmt is {:?}", result);
/// ```

#[derive(Debug)]
pub struct InterestPayment {
    rate: f64,
    per: u32,
    nper: u32,
    pv: f64,
    fv: f64,
    when: WhenType,
}

impl InterestPayment {
    /// Instantiate a `InterestPayment` instance from a tuple of (`rate`, `per`, `nper`, `pv`, `fv` and `when`) in said order
    pub fn from_tuple(tup: (f64, u32, u32, f64, f64, WhenType)) -> Result<Self> {
        Ok(InterestPayment {
            rate: tup.0,
            per: tup.1,
            nper: tup.2,
            pv: tup.3,
            fv: tup.4,
            when: tup.5,
        })
    }

    /// Instantiate a `InterestPayment` instance from a hash map with keys of (`rate`, `per`, `nper`, `pv` and `when`) in said order
    /// Since [`HashMap`] requires values of same type, we need to wrap into a variant of enum
    pub fn from_map(map: ParaMap) -> Result<Self> {
        let op = |err: Error| {
            Error::OtherError(format!(
                "Failed construct an instance of `InterestPayment` from: `{:?}` <- {}",
                map, err
            ))
        };

        let rate = get_f64(&map, "rate").map_err(&op)?;
        let per = get_u32(&map, "per").map_err(&op)?;
        let nper = get_u32(&map, "nper").map_err(&op)?;
        let pv = get_f64(&map, "pv").map_err(&op)?;
        let fv = get_f64(&map, "fv").map_err(&op)?;
        let when = get_when(&map, "when").map_err(op)?;
        Ok(InterestPayment {
            rate,
            per,
            nper,
            pv,
            fv,
            when,
        })
    }

    fn ipmt(&self) -> Result<Option<f64>> {
        ipmt(self.rate, self.per, self.nper, self.pv, self.fv, self.when)
    }

    /// Get the interet payment from an instance of `InterestPayment`
    pub fn get(&self) -> Result<Option<f64>> {
        self.ipmt()
    }
}

impl FromTuple<(f64, u32, u32, f64, f64, WhenType)> for InterestPayment {
    fn from_tuple(tup: (f64, u32, u32, f64, f64, WhenType)) -> Result<Self> {
        InterestPayment::from_tuple(tup)
    }
}

impl FromMap for InterestPayment {
    fn from_map(map: ParaMap) -> Result<Self> {
        InterestPayment::from_map(map)
    }
}

/// Compute the interest portion of a payment
/// ## Parameters
/// * `rate` : an interest rate compounded once per period
/// * `per` : the payment period to calculate the interest amount
/// * `nper` : number of compounding periods
/// * `pv` : a present value
/// * `fv` : a future value
/// * `when` : when payments are due [`WhenType`]
///
/// ## Return:
/// * the interest portion in a payment or `None`
///
/// ## Example
/// ```rust
/// use rfinancial::*;
/// let result = ipmt(0.1 / 12.0, 1, 24, 2000.0, 0.0, WhenType::End).expect("Error computing ipmt");
/// println!("ipmt is {:?}", result);
/// ```
pub fn ipmt(
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
    // remaining balance
    // only consider per > 1, i.e. starting from 1st payment
    let impt = if per >= 1 {
        // fv is imported as `future_value` since `fv` is masked by the argument `fv`
        let rbl = future_value(rate, per - 1, total_pmt, pv, when)?;

        match when {
            WhenType::Begin => {
                if per == 1 {
                    // if payment is made at begin of a period, interest portion is 0 for 1st payment
                    Some(0.0)
                } else {
                    // discount for 2nd payment and beyond
                    Some(rbl / (1.0 + rate) * rate)
                }
            }
            WhenType::End => Some(rbl * rate),
        }
        // if 0th or negative-th(not possible though since u32) payments are requested, return None
    } else {
        None
    };

    Ok(impt)
}

/// Compute the interest portion of a payment from a hash map with keys of (`rate`, `per`, `nper`, `pv`, `fv`, and `when`)
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
/// let result = ipmt_from_map(map).expect("Error computing ipmt");
/// println!("ipmt is {:?}", result);
/// ```
pub fn ipmt_from_map(map: ParaMap) -> Result<Option<f64>> {
    let op = |err: Error| {
        Error::OtherError(format!(
            "Failed to compute `ipmt` from: `{:?}` <- {}",
            map, err
        ))
    };

    let rate = get_f64(&map, "rate").map_err(&op)?;
    let per = get_u32(&map, "per").map_err(&op)?;
    let nper = get_u32(&map, "nper").map_err(&op)?;
    let pv = get_f64(&map, "pv").map_err(&op)?;
    let fv = get_f64(&map, "fv").map_err(&op)?;
    let when = get_when(&map, "when").map_err(op)?;
    ipmt(rate, per, nper, pv, fv, when)
}

/// Builder for constructing `InterestPayment` instances with a fluent API
///
/// Provides sensible defaults for optional parameters:
/// - `when`: `WhenType::End` (payments due at end of period)
///
/// ## Example
/// ```rust
/// use rfinancial::*;
/// let ipmt = InterestPaymentBuilder::new()
///     .rate(0.1 / 12.0)
///     .per(1)
///     .nper(24)
///     .pv(2000.0)
///     .build()?;
/// println!("ipmt is {:?}", ipmt.get());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone)]
pub struct InterestPaymentBuilder {
    rate: f64,
    per: u32,
    nper: u32,
    pv: f64,
    fv: f64,
    when: WhenType,
}

impl InterestPaymentBuilder {
    /// Create a new `InterestPaymentBuilder` with default values
    pub fn new() -> Self {
        InterestPaymentBuilder {
            rate: 0.0,
            per: 1,
            nper: 1,
            pv: 0.0,
            fv: 0.0,
            when: WhenType::End,
        }
    }

    /// Set the interest rate compounded once per period
    pub fn rate(mut self, rate: f64) -> Self {
        self.rate = rate;
        self
    }

    /// Set the payment period to calculate interest for
    pub fn per(mut self, per: u32) -> Self {
        self.per = per;
        self
    }

    /// Set the number of compounding periods
    pub fn nper(mut self, nper: u32) -> Self {
        self.nper = nper;
        self
    }

    /// Set the present value
    pub fn pv(mut self, pv: f64) -> Self {
        self.pv = pv;
        self
    }

    /// Set the future value (defaults to 0.0)
    pub fn fv(mut self, fv: f64) -> Self {
        self.fv = fv;
        self
    }

    /// Set when payments are due (defaults to `WhenType::End`)
    pub fn when(mut self, when: WhenType) -> Self {
        self.when = when;
        self
    }

    /// Build the `InterestPayment` instance
    pub fn build(self) -> Result<InterestPayment> {
        Ok(InterestPayment {
            rate: self.rate,
            per: self.per,
            nper: self.nper,
            pv: self.pv,
            fv: self.fv,
            when: self.when,
        })
    }
}

impl Default for InterestPaymentBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_ipmt_from_tuple() {
        let ipmt =
            InterestPayment::from_tuple((0.1 / 12.0, 1, 24, 2000.0, 0.0, WhenType::End)).unwrap();
        let cond = (ipmt.rate == 0.1 / 12.0)
            && (ipmt.per == 1)
            && (ipmt.nper == 24)
            && (ipmt.pv == 2000.0)
            && (ipmt.fv == 0.0)
            && (ipmt.when == WhenType::End);

        assert!(cond);
    }

    #[test]
    fn test_ipmt_from_map() {
        let mut map = ParaMap::new();
        map.insert("rate".into(), ParaType::F64(0.1 / 12.0));
        map.insert("per".into(), ParaType::U32(1));
        map.insert("nper".into(), ParaType::U32(24));
        map.insert("pv".into(), ParaType::F64(2000.0));
        map.insert("fv".into(), ParaType::F64(0.0));
        map.insert("when".into(), ParaType::When(WhenType::End));
        let ipmt = InterestPayment::from_map(map).unwrap();
        let cond = (ipmt.rate == 0.1 / 12.0)
            && (ipmt.per == 1)
            && (ipmt.nper == 24)
            && (ipmt.pv == 2000.0)
            && (ipmt.fv == 0.0)
            && (ipmt.when == WhenType::End);

        assert!(cond);
    }

    #[test]
    fn test_ipmt_with_end() {
        let rate = 0.1 / 12.0;
        let per = 1;
        let nper = 24;
        let pv = 2000.0;
        let fv = 0.0;
        let when = WhenType::End;

        let ipmt = InterestPayment {
            rate,
            per,
            nper,
            pv,
            fv,
            when,
        };
        // npf.ipmt(0.1 / 12, 1, 24, 2000),
        // -16.666667
        let res = ipmt.get().unwrap().unwrap();
        let tgt = -16.666667;
        assert!(
            float_close(res, tgt, RTOL, ATOL),
            "{:#?} v.s. {:#?}",
            res,
            tgt
        );
    }

    #[test]
    fn test_ipmt_with_begin_1() {
        let rate = 0.0824 / 12.0;
        let per = 1;
        let nper = 12;
        let pv = 2500.0;
        let fv = 0.0;
        let when = WhenType::Begin;

        let ipmt = InterestPayment {
            rate,
            per,
            nper,
            pv,
            fv,
            when,
        };
        // npf.ipmt(0.0824 / 12, 1, 12, 2500, 0, 'begin')
        // array(0.)
        let res = ipmt.get().unwrap().unwrap();
        let tgt = 0.0;
        assert!(
            float_close(res, tgt, RTOL, ATOL),
            "{:#?} v.s. {:#?}",
            res,
            tgt
        );
    }

    #[test]
    fn test_ipmt_with_begin_2() {
        let rate = 0.0824 / 12.0;
        let per = 2;
        let nper = 12;
        let pv = 2500.0;
        let fv = 0.0;
        let when = WhenType::Begin;

        let ipmt = InterestPayment {
            rate,
            per,
            nper,
            pv,
            fv,
            when,
        };
        // npf.ipmt(0.0824 / 12, 2, 12, 2500, 0, 'begin')
        // array(-15.68165675)
        let res = ipmt.get().unwrap().unwrap();
        let tgt = -15.68165675;
        assert!(
            float_close(res, tgt, RTOL, ATOL),
            "{:#?} v.s. {:#?}",
            res,
            tgt
        );
    }

    #[test]
    fn test_ipmt_zero_per() {
        let rate = 0.1 / 12.0;
        let per = 0;
        let nper = 24;
        let pv = 2000.0;
        let fv = 0.0;
        let when = WhenType::End;

        let ipmt = InterestPayment {
            rate,
            per,
            nper,
            pv,
            fv,
            when,
        };
        let res = ipmt.get().unwrap();
        let tgt = None;
        assert_eq!(res, tgt, "{:#?} v.s. {:#?}", res, tgt);
    }

    #[test]
    fn test_ipmt_nan() {
        let mut map = ParaMap::new();
        map.insert("rate".into(), ParaType::F64(0.1 / 12.0));
        map.insert("per".into(), ParaType::U32(1));
        map.insert("nper".into(), ParaType::U32(u32::MAX));
        map.insert("pv".into(), ParaType::F64(2000.0));
        map.insert("fv".into(), ParaType::F64(0.0));
        map.insert("when".into(), ParaType::When(WhenType::End));
        let ipmt = InterestPayment::from_map(map).unwrap();
        let cond = ipmt.get().unwrap().unwrap().is_nan();

        assert!(cond);
    }

    #[test]
    fn test_ipmt_err() {
        let mut map = ParaMap::new();
        map.insert("Rate".into(), ParaType::F64(0.1 / 12.0));
        map.insert("per".into(), ParaType::U32(1));
        map.insert("nper".into(), ParaType::U32(u32::MAX));
        map.insert("pv".into(), ParaType::F64(2000.0));
        map.insert("fv".into(), ParaType::F64(0.0));
        map.insert("when".into(), ParaType::When(WhenType::End));
        let ipmt = InterestPayment::from_map(map);
        let cond = ipmt.is_err();

        assert!(cond);
    }

    #[test]
    fn test_ipmt_function() {
        // npf.ipmt(0.1 / 12, 1, 24, 2000),
        // -16.666667
        let res = super::ipmt(0.1 / 12.0, 1, 24, 2000.0, 0.0, WhenType::End)
            .unwrap()
            .unwrap();
        let tgt = -16.666667;
        assert!(
            float_close(res, tgt, RTOL, ATOL),
            "{:#?} v.s. {:#?}",
            res,
            tgt
        );
    }

    #[test]
    fn test_ipmt_from_map_function() {
        let mut map = ParaMap::new();
        map.insert("rate".into(), ParaType::F64(0.1 / 12.0));
        map.insert("per".into(), ParaType::U32(1));
        map.insert("nper".into(), ParaType::U32(24));
        map.insert("pv".into(), ParaType::F64(2000.0));
        map.insert("fv".into(), ParaType::F64(0.0));
        map.insert("when".into(), ParaType::When(WhenType::End));

        // npf.ipmt(0.1 / 12, 1, 24, 2000),
        // -16.666667
        let res = super::ipmt_from_map(map).unwrap().unwrap();
        let tgt = -16.666667;
        assert!(
            float_close(res, tgt, RTOL, ATOL),
            "{:#?} v.s. {:#?}",
            res,
            tgt
        );
    }

    #[test]
    fn test_ipmt_builder() {
        let ipmt = InterestPaymentBuilder::new()
            .rate(0.1 / 12.0)
            .per(1)
            .nper(24)
            .pv(2000.0)
            .fv(0.0)
            .when(WhenType::End)
            .build()
            .unwrap();

        let res = ipmt.get().unwrap().unwrap();
        let tgt = -16.666667;
        assert!(
            float_close(res, tgt, RTOL, ATOL),
            "{:#?} v.s. {:#?}",
            res,
            tgt
        );
    }

    #[test]
    fn test_ipmt_builder_with_defaults() {
        // Test that builder uses sensible defaults for fv and when
        let ipmt = InterestPaymentBuilder::new()
            .rate(0.1 / 12.0)
            .per(1)
            .nper(24)
            .pv(2000.0)
            .build()
            .unwrap();

        let res = ipmt.get().unwrap().unwrap();
        let tgt = -16.666667;
        assert!(
            float_close(res, tgt, RTOL, ATOL),
            "{:#?} v.s. {:#?}",
            res,
            tgt
        );
    }
}
