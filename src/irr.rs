use crate::{Error, FromMap, FromTuple, ParaMap, Result, get_vecf64};
/// # Compute the Internal Rate of Return (IRR)
/// This is the "average" periodically compounded rate of return that gives a net present value of 0.0
/// ## Parameters
/// `values` : array_like, shape(N,)
/// * input cash flows per time period
/// * by convention, net "deposits" are negative and net "withdrawals" are positive
/// * e.g., the first element of `values`, which represents the initial investment, is typically negative
/// ## Return
/// * `irr`: internal rate of return for periodic input `values`
///
/// ## Example
/// Struct-based:
/// ```rust
/// use rfinancial::*;
/// let values: Vec<f64> = vec![-150000.0, 15000.0, 25000.0, 35000.0, 45000.0, 60000.0];
/// let result = InternalRateReturn::from_vec(values).expect("Error creating IRR");
/// println!("{:#?}'s irr is {:?}", result, result.get());
/// ```
/// Function-based:
/// ```rust
/// use rfinancial::*;
/// let values: Vec<f64> = vec![-150000.0, 15000.0, 25000.0, 35000.0, 45000.0, 60000.0];
/// let result = irr(&values).expect("Error computing irr");
/// println!("irr is {:?}", result);
/// ```
/// ## Caveat
/// * I use Newton-Raphson method to find first `irr` that makes the `npv` of given cash flows 0
/// * I am still trying to find/craft packge to find roots of polynomial in similar way as `numpy_financial`
/// * Appreciate any feedbacks
#[derive(Debug)]
pub struct InternalRateReturn {
    values: Vec<f64>,
}

impl InternalRateReturn {
    /// Instantiate an `InternalRateReturn` instance from a vector of `f64`
    ///
    /// # Errors
    /// Returns `ParaError` if:
    /// - `values` has fewer than 2 elements
    /// - `values` contains only positive or only negative values (IRR requires both)
    pub fn from_vec(values: Vec<f64>) -> Result<Self> {
        validate_values(&values)?;
        Ok(InternalRateReturn { values })
    }

    /// Instantiate a `InterestPayment` instance from a hash map with keys of (`values`)
    /// Since [`HashMap`] requires values of same type, we need to wrap into a variant of enum
    ///
    /// # Errors
    /// Returns error if values cannot be extracted from map or validation fails
    pub fn from_map(map: ParaMap) -> Result<Self> {
        let op = |err: Error| {
            Error::OtherError(format!(
                "Failed construct an instance of `InternalRateReturn` from: `{:?}` <- {}",
                map, err
            ))
        };
        let values = get_vecf64(&map, "values").map_err(op)?;
        Self::from_vec(values)
            .map_err(|e| Error::OtherError(format!("IRR validation failed: {}", e)))
    }

    /// Raw polynomial `sum(v[i] * x^(n-1-i))`. Retained to verify roots in tests; the solver
    /// itself uses the overflow-resistant [`Self::gx`] form.
    #[cfg(test)]
    fn fx(v: &[f64], x: f64) -> Result<f64> {
        let fx: f64 = v
            .iter()
            .rev()
            .enumerate()
            .map(|(p, c)| c * x.powf(p as f64))
            .sum();
        Ok(fx)
    }

    #[cfg(test)]
    fn dx(v: &[f64], x: f64) -> Result<f64> {
        let dx: f64 = v
            .iter()
            .rev()
            .skip(1)
            .enumerate()
            .map(|(p, c)| {
                let p = p as f64;
                c * (p + 1.0) * x.powf(p)
            })
            .sum();
        Ok(dx)
    }

    /// Evaluate `sum(v[i] / x^i)` by Horner's method, where `x = 1 + rate`.
    ///
    /// This is the NPV form of the cash flow polynomial. It shares the same roots as the raw
    /// polynomial for `x > 0` (they differ by the strictly positive factor `x^(n-1)`), but the
    /// terms shrink instead of growing, so it stays finite for long series where the raw form
    /// would overflow to infinity.
    fn gx(v: &[f64], x: f64) -> f64 {
        v.iter().rev().fold(0.0, |acc, &c| acc / x + c)
    }

    /// Evaluate `gx` together with `sum(i * v[i] / x^i)`, whose sign is the negation of `gx`'s
    /// slope for `x > 0`.
    ///
    /// The slope only serves to detect that a cell contains a local extremum, which is what
    /// reveals a pair of roots too close together to change the sign at the cell endpoints.
    /// Both come from one pass over the cash flows.
    fn gx_and_slope(v: &[f64], x: f64) -> (f64, f64) {
        v.iter()
            .enumerate()
            .rev()
            .fold((0.0, 0.0), |(g, s), (i, &c)| {
                (g / x + c, s / x + (i as f64) * c)
            })
    }

    /// Find the IRR root as `x = 1 + rate`.
    ///
    /// Only `x > 0` is economically meaningful (a rate above -100%). Non-conventional cash flows
    /// can still have several such roots, so every sign change in range is bracketed and
    /// refined, and the one closest to a zero rate is returned. This matches how
    /// `numpy_financial` selects among multiple IRRs.
    ///
    /// Bracketing then bisecting cannot latch onto a negative root or diverge the way an
    /// unguarded Newton iteration can. Unlike an all-roots eigenvalue solve, a scan can in
    /// principle still miss a closely spaced pair; [`Self::refine`] is what makes that unlikely.
    fn find_root(v: &[f64]) -> Result<Option<f64>> {
        // x in [1e-6, 1e3] covers rates from -99.9999% to +99900%.
        const LO: f64 = 1e-6;
        const HI: f64 = 1e3;
        const STEPS: i32 = 512;
        const MAX_DEPTH: u32 = 16;

        let ratio = (HI / LO).powf(1.0 / STEPS as f64);

        let mut roots: Vec<f64> = Vec::new();
        let mut lo = LO;
        let (mut f_lo, mut s_lo) = Self::gx_and_slope(v, lo);
        if f_lo == 0.0 {
            roots.push(lo);
        }

        for i in 1..=STEPS {
            let hi = LO * ratio.powi(i);
            let (f_hi, s_hi) = Self::gx_and_slope(v, hi);

            if f_hi == 0.0 {
                roots.push(hi);
            } else if f_lo != 0.0 && sign_differs(f_lo, f_hi) {
                roots.push(Self::bisect(v, lo, f_lo, hi));
            } else if f_lo != 0.0 {
                Self::refine(v, lo, f_lo, s_lo, hi, f_hi, s_hi, MAX_DEPTH, &mut roots);
            }

            lo = hi;
            f_lo = f_hi;
            s_lo = s_hi;
        }

        // Closest to a zero rate, i.e. to x = 1. Returns `None` when no sign change was found.
        Ok(roots.into_iter().min_by(|a, b| {
            (a - 1.0)
                .abs()
                .partial_cmp(&(b - 1.0).abs())
                .expect("root distances are finite")
        }))
    }

    /// Search inside a cell whose endpoints share a sign.
    ///
    /// An even number of roots in one cell leaves the endpoint signs equal, so a plain scan
    /// misses them however fine the grid is. Two roots must straddle a local extremum, so a
    /// slope sign change in the cell is the reliable signal to subdivide; the `|f|` dip is a
    /// cheap secondary cue. Monotone stretches trigger neither and cost nothing extra.
    #[allow(clippy::too_many_arguments)]
    fn refine(
        v: &[f64],
        lo: f64,
        f_lo: f64,
        s_lo: f64,
        hi: f64,
        f_hi: f64,
        s_hi: f64,
        depth: u32,
        roots: &mut Vec<f64>,
    ) {
        if depth == 0 {
            return;
        }
        // Two roots in one cell must straddle a local extremum, which turns the slope.
        if !sign_differs(s_lo, s_hi) {
            return;
        }

        // Geometric midpoint, matching the geometric grid.
        let mid = (lo * hi).sqrt();
        if mid <= lo || mid >= hi {
            return;
        }

        let (f_mid, s_mid) = Self::gx_and_slope(v, mid);
        if f_mid == 0.0 {
            roots.push(mid);
            return;
        }

        if sign_differs(f_lo, f_mid) {
            roots.push(Self::bisect(v, lo, f_lo, mid));
        } else {
            Self::refine(v, lo, f_lo, s_lo, mid, f_mid, s_mid, depth - 1, roots);
        }

        if sign_differs(f_mid, f_hi) {
            roots.push(Self::bisect(v, mid, f_mid, hi));
        } else {
            Self::refine(v, mid, f_mid, s_mid, hi, f_hi, s_hi, depth - 1, roots);
        }
    }

    /// Bisect a bracketed sign change down to the limit of `f64` resolution.
    fn bisect(v: &[f64], mut lo: f64, mut f_lo: f64, mut hi: f64) -> f64 {
        for _ in 0..200 {
            let mid = 0.5 * (lo + hi);
            // No representable point strictly between lo and hi.
            if mid <= lo || mid >= hi {
                break;
            }
            let f_mid = Self::gx(v, mid);
            if f_mid == 0.0 {
                return mid;
            }
            if sign_differs(f_lo, f_mid) {
                hi = mid;
            } else {
                lo = mid;
                f_lo = f_mid;
            }
        }
        0.5 * (lo + hi)
    }

    fn irr(&self) -> Result<Option<f64>> {
        irr(&self.values)
    }

    /// Get the `irr` from an instance of `InternalRateReturn`
    pub fn get(&self) -> Result<Option<f64>> {
        self.irr()
    }
}

impl FromTuple<Vec<f64>> for InternalRateReturn {
    fn from_tuple(values: Vec<f64>) -> Result<Self> {
        InternalRateReturn::from_vec(values)
    }
}

impl FromMap for InternalRateReturn {
    fn from_map(map: ParaMap) -> Result<Self> {
        InternalRateReturn::from_map(map)
    }
}

/// Compare signs directly; testing `a * b < 0.0` can overflow or underflow to zero.
fn sign_differs(a: f64, b: f64) -> bool {
    (a < 0.0) != (b < 0.0)
}

/// Validate that `values` has at least 2 elements and contains both positive and negative values
fn validate_values(values: &[f64]) -> Result<()> {
    if values.len() < 2 {
        return Err(Error::ParaError(
            "values must contain at least 2 elements".to_string(),
        ));
    }

    let has_positive = values.iter().any(|&v| v > 0.0);
    let has_negative = values.iter().any(|&v| v < 0.0);

    if !has_positive || !has_negative {
        return Err(Error::ParaError(
            "values must contain both positive and negative values for IRR to exist".to_string(),
        ));
    }

    Ok(())
}

/// Compute the Internal Rate of Return (IRR)
/// ## Parameters
/// `values` : array_like, shape(N,)
/// * input cash flows per time period
/// * by convention, net "deposits" are negative and net "withdrawals" are positive
/// * e.g., the first element of `values`, which represents the initial investment, is typically negative
/// ## Return
/// * internal rate of return for periodic input `values`
///
/// # Errors
/// Returns `ParaError` if:
/// - `values` has fewer than 2 elements
/// - `values` contains only positive or only negative values (IRR requires both)
///
/// Returns `Ok(None)` if no internal rate of return exists for `values`.
///
/// ## Example
/// ```rust
/// use rfinancial::*;
/// let values: Vec<f64> = vec![-150000.0, 15000.0, 25000.0, 35000.0, 45000.0, 60000.0];
/// let result = irr(&values).expect("Error computing irr");
/// println!("irr is {:?}", result);
/// ```
pub fn irr(values: &[f64]) -> Result<Option<f64>> {
    validate_values(values)?;
    Ok(InternalRateReturn::find_root(values)?.map(|x| x - 1.0))
}

/// Compute the Internal Rate of Return (IRR) from a hash map with key `values`
/// Since [`HashMap`] requires values of same type, we need to wrap into a variant of enum
/// ## Example
/// ```rust
/// use rfinancial::*;
/// let values: Vec<f64> = vec![-150000.0, 15000.0, 25000.0, 35000.0, 45000.0, 60000.0];
/// let mut map = ParaMap::new();
/// map.insert("values".into(), ParaType::VecF64(values));
/// let result = irr_from_map(map).expect("Error computing irr");
/// println!("irr is {:?}", result);
/// ```
pub fn irr_from_map(map: ParaMap) -> Result<Option<f64>> {
    let op = |err: Error| {
        Error::OtherError(format!(
            "Failed to compute `irr` from: `{:?}` <- {}",
            map, err
        ))
    };

    let values = get_vecf64(&map, "values").map_err(op)?;
    irr(&values)
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_irr_fx() {
        let c: Vec<f64> = vec![1.0, 2.0, 3.0];
        let x = 2.0;
        let res = InternalRateReturn::fx(&c, x).unwrap();
        // 1*x^2 + 2*x^1 + 3*x^0 ->
        // 1*2^2 + 2*2^1 + 3*2^0 -> 11
        let tgt = 11.0;
        assert_eq!(res, tgt, "{} v.s. {}", res, tgt);
    }

    #[test]
    fn test_irr_dx() {
        let c: Vec<f64> = vec![1.0, 2.0, 3.0];
        let x = 2.0;
        let res = InternalRateReturn::dx(&c, x).unwrap();
        // 1*x^2 + 2*x^1 + 3*x^0 ->
        // 1*2*x^1 + 2*1*x^0 + 0 ->
        // 1*2*2^1 + 2*1*2^0 + 0 ->
        let tgt = 6.0;
        assert_eq!(res, tgt, "{} v.s. {}", res, tgt);
    }

    #[test]
    fn test_irr_find_root() {
        // -1.0 * x^2 + 1=0 -> x =1 and -1
        // let c: Vec<f64> = vec![-1.0, 0.0, 1.0];

        // - 0.25* x^2 + 1=0 -> x =2 and -2
        let c: Vec<f64> = vec![-0.25, 0.0, 1.0];

        let root = InternalRateReturn::find_root(&c).unwrap().unwrap();
        let tgt = InternalRateReturn::fx(&c, root).unwrap();
        let res = 0.0;
        assert!(
            float_close(res, tgt, RTOL, ATOL),
            "{:#?} v.s. {:#?}",
            res,
            tgt
        )
    }

    #[test]
    fn test_irr_from_vec() {
        // npf.irr([-150000, 15000, 25000, 35000, 45000, 60000])
        // 0.052432888859413884
        let values: Vec<f64> = vec![-150000.0, 15000.0, 25000.0, 35000.0, 45000.0, 60000.0];
        let res = InternalRateReturn::from_vec(values)
            .unwrap()
            .get()
            .unwrap()
            .unwrap();
        let tgt = 0.052432888859413884;
        assert!(
            float_close(res, tgt, RTOL, ATOL),
            "{:#?} v.s. {:#?}",
            res,
            tgt
        )
    }

    #[test]
    fn test_irr_from_map() {
        // npf.irr([-150000, 15000, 25000, 35000, 45000, 60000])
        // 0.052432888859413884
        let values: Vec<f64> = vec![-150000.0, 15000.0, 25000.0, 35000.0, 45000.0, 60000.0];
        let mut map = ParaMap::new();
        map.insert("values".to_string(), ParaType::VecF64(values));
        let res = InternalRateReturn::from_map(map)
            .unwrap()
            .get()
            .unwrap()
            .unwrap();
        let tgt = 0.052432888859413884;
        assert!(
            float_close(res, tgt, RTOL, ATOL),
            "{:#?} v.s. {:#?}",
            res,
            tgt
        )
    }

    #[test]
    fn test_irr_err() {
        let values: Vec<f64> = vec![-150000.0, 15000.0, 25000.0, 35000.0, 45000.0, 60000.0];
        let mut map = ParaMap::new();
        map.insert("Values".to_string(), ParaType::VecF64(values));
        let res = InternalRateReturn::from_map(map);
        let cond = res.is_err();
        assert!(cond);
    }

    /// A level annuity has an IRR equal to its rate by construction. The previous solver seeded
    /// Newton at a fixed -0.9 and returned the first root it reached, which gave a spurious
    /// negative root for odd-length series and panicked at length 16.
    #[test]
    fn test_irr_annuity_known_rate() {
        for len in [4usize, 7, 15, 16, 17, 64, 257, 1024] {
            let n = (len - 1) as f64;
            let principal = 10_000.0_f64;
            let rate = 0.08_f64;
            let payment = principal * rate / (1.0 - (1.0 + rate).powf(-n));

            let mut values = vec![-principal];
            values.extend(std::iter::repeat(payment).take(len - 1));

            let res = super::irr(&values).unwrap().unwrap();
            assert!(
                float_close(res, rate, RTOL, ATOL),
                "len {}: {} v.s. {}",
                len,
                res,
                rate
            )
        }
    }

    /// IRR must be the positive-`x` root, i.e. a rate above -100%.
    #[test]
    fn test_irr_above_negative_one() {
        let values: Vec<f64> = vec![-1000.0, 100.0, 100.0, 100.0];
        let res = super::irr(&values).unwrap().unwrap();
        assert!(res > -1.0, "irr {} must be > -1.0", res);
        assert!(
            float_close(npv(&values, res).unwrap(), 0.0, RTOL, ATOL),
            "npv at irr {} is not zero",
            res
        )
    }

    /// Cases from the `numpy_financial` 1.0.0 docstring. The last two are non-conventional
    /// series with several positive roots, where the selection rule decides the answer.
    #[test]
    fn test_irr_matches_numpy_financial_cases() {
        let cases: Vec<(Vec<f64>, f64)> = vec![
            (vec![-100.0, 39.0, 59.0, 55.0, 20.0], 0.28095),
            (vec![-100.0, 0.0, 0.0, 74.0], -0.0955),
            (vec![-100.0, 100.0, 0.0, -7.0], -0.0833),
            (vec![-100.0, 100.0, 0.0, 7.0], 0.06206),
            (vec![-5.0, 10.5, 1.0, -8.0, 1.0], 0.0886),
        ];

        for (values, tgt) in cases {
            let res = super::irr(&values).unwrap().unwrap();
            assert!(
                (res - tgt).abs() < 5e-6,
                "{:?}: {} v.s. {}",
                values,
                res,
                tgt
            )
        }
    }

    #[test]
    fn test_irr_function() {
        // npf.irr([-150000, 15000, 25000, 35000, 45000, 60000])
        // 0.052432888859413884
        let values: Vec<f64> = vec![-150000.0, 15000.0, 25000.0, 35000.0, 45000.0, 60000.0];
        let res = super::irr(&values).unwrap().unwrap();
        let tgt = 0.052432888859413884;
        assert!(
            float_close(res, tgt, RTOL, ATOL),
            "{:#?} v.s. {:#?}",
            res,
            tgt
        )
    }

    #[test]
    fn test_irr_from_map_function() {
        // npf.irr([-150000, 15000, 25000, 35000, 45000, 60000])
        // 0.052432888859413884
        let values: Vec<f64> = vec![-150000.0, 15000.0, 25000.0, 35000.0, 45000.0, 60000.0];
        let mut map = ParaMap::new();
        map.insert("values".to_string(), ParaType::VecF64(values));
        let res = super::irr_from_map(map).unwrap().unwrap();
        let tgt = 0.052432888859413884;
        assert!(
            float_close(res, tgt, RTOL, ATOL),
            "{:#?} v.s. {:#?}",
            res,
            tgt
        )
    }
}
