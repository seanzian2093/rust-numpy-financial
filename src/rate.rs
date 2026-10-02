use crate::{
    Error, FromMap, FromTuple, ParaMap, Result, get_f64, get_u32, get_when, util::WhenType,
};
use log::debug;
/// # Compute the interest rate
/// ## Parameters
/// * `nper` : number of compounding periods
/// * `pmt` : payment in each period
/// * `pv` : present value
/// * `fv`: the value at the end of the `nper` periods
/// * `when` : when payments are due [`WhenType`]. Defaults to `When::End`
/// * `guess` : starting guess for solving the rate of interest
/// * `tol` : required tolerance for the solution
/// * `maxiter` : maximum iterations in finding the solution
///
/// ## Return:
/// * `rate` : an interest rate compounded once per period or `None`
///
/// ## Example
/// Struct-based:
/// ```rust
/// use rfinancial::*;
/// let result = Rate::from_tuple((10, 0.0, -3500.0, 10000.0, WhenType::End, 0.1, 1e-6, 100)).expect("Error creating Rate");
/// println!("{:#?}'s rate is {:#?}", result, result.get());
/// ```
/// Function-based:
/// ```rust
/// use rfinancial::*;
/// let result = rate(10, 0.0, -3500.0, 10000.0, WhenType::End, 0.1, 1e-6, 100).expect("Error computing rate");
/// println!("rate is {:#?}", result);
/// ```
///
#[derive(Debug)]
pub struct Rate {
    nper: u32,
    pmt: f64,
    pv: f64,
    fv: f64,
    when: WhenType,
    guess: f64,
    tol: f64,
    maxiter: u32,
}

impl Rate {
    /// Instantiate a `Rate` instance from a tuple of (`nper`, `pmt`, `pv`, `fv`, `when`, `guess`, `tol`, `maxiter`) in said order
    pub fn from_tuple(tup: (u32, f64, f64, f64, WhenType, f64, f64, u32)) -> Result<Self> {
        Ok(Rate {
            nper: tup.0,
            pmt: tup.1,
            pv: tup.2,
            fv: tup.3,
            when: tup.4,
            guess: tup.5,
            tol: tup.6,
            maxiter: tup.7,
        })
    }

    /// Instantiate a `Rate` instance from a hash map with keys of (`nper`, `pmt`, `pv`, `fv`, `when`, `guess`, `tol`, `maxiter`) in said order
    /// Since [`HashMap`] requires values of same type, we need to wrap into a variant of enum
    pub fn from_map(map: ParaMap) -> Result<Self> {
        let op = |err: Error| {
            Error::OtherError(format!(
                "Failed construct an instance of `Rate` from: `{:?}` <- {}",
                map, err
            ))
        };

        let nper = get_u32(&map, "nper").map_err(&op)?;
        let pmt = get_f64(&map, "pmt").map_err(&op)?;
        let pv = get_f64(&map, "pv").map_err(&op)?;
        let fv = get_f64(&map, "fv").map_err(&op)?;
        let when = get_when(&map, "when").map_err(&op)?;
        let guess = get_f64(&map, "guess").map_err(&op)?;
        let tol = get_f64(&map, "tol").map_err(&op)?;
        let maxiter = get_u32(&map, "maxiter").map_err(op)?;
        Ok(Rate {
            nper,
            pmt,
            pv,
            fv,
            when,
            guess,
            tol,
            maxiter,
        })
    }

    /// Evaluate `g(r_n)/g'(r_n)`, where `g = fv + pv*(1+rate)**nper + pmt*(1+rate*when)/rate * ((1+rate)**nper - 1)`
    fn _g_div_gp(r: f64, n: u32, p: f64, x: f64, y: f64, w: WhenType) -> f64 {
        // converts to f64 for calculation
        let n = n as f64;
        let w = w as u8 as f64;

        let t1 = (r + 1.0).powf(n);
        let t2 = (r + 1.0).powf(n - 1.0);
        let g = y + t1 * x + p * (t1 - 1.0) * (r * w + 1.0) / r;
        let gp = n * t2 * x - p * (t1 - 1.0) * (r * w + 1.0) / (r.powf(2.0))
            + n * p * t2 * (r * w + 1.0) / r
            + p * (t1 - 1.0) * w / r;
        g / gp
    }

    fn rate(&self) -> Result<Option<f64>> {
        rate(
            self.nper,
            self.pmt,
            self.pv,
            self.fv,
            self.when,
            self.guess,
            self.tol,
            self.maxiter,
        )
    }

    /// Get the rate from an instance of `Rate`
    pub fn get(&self) -> Result<Option<f64>> {
        self.rate()
    }
}

impl FromTuple<(u32, f64, f64, f64, WhenType, f64, f64, u32)> for Rate {
    fn from_tuple(tup: (u32, f64, f64, f64, WhenType, f64, f64, u32)) -> Result<Self> {
        Rate::from_tuple(tup)
    }
}

impl FromMap for Rate {
    fn from_map(map: ParaMap) -> Result<Self> {
        Rate::from_map(map)
    }
}

/// Evaluate `g(r_n)/g'(r_n)`, where `g = fv + pv*(1+rate)**nper + pmt*(1+rate*when)/rate * ((1+rate)**nper - 1)`
fn g_div_gp(r: f64, n: u32, p: f64, x: f64, y: f64, w: WhenType) -> f64 {
    // converts to f64 for calculation
    let n = n as f64;
    let w = w as u8 as f64;

    let t1 = (r + 1.0).powf(n);
    let t2 = (r + 1.0).powf(n - 1.0);
    let g = y + t1 * x + p * (t1 - 1.0) * (r * w + 1.0) / r;
    let gp = n * t2 * x - p * (t1 - 1.0) * (r * w + 1.0) / (r.powf(2.0))
        + n * p * t2 * (r * w + 1.0) / r
        + p * (t1 - 1.0) * w / r;
    g / gp
}

/// Compute the interest rate
/// ## Parameters
/// * `nper` : number of compounding periods
/// * `pmt` : payment in each period
/// * `pv` : present value
/// * `fv`: the value at the end of the `nper` periods
/// * `when` : when payments are due [`WhenType`]
/// * `guess` : starting guess for solving the rate of interest
/// * `tol` : required tolerance for the solution
/// * `maxiter` : maximum iterations in finding the solution
///
/// ## Return:
/// * an interest rate compounded once per period or `None`
///
/// ## Example
/// ```rust
/// use rfinancial::*;
/// let result = rate(10, 0.0, -3500.0, 10000.0, WhenType::End, 0.1, 1e-6, 100).expect("Error computing rate");
/// println!("rate is {:#?}", result);
/// ```
#[allow(clippy::too_many_arguments)]
pub fn rate(
    nper: u32,
    pmt: f64,
    pv: f64,
    fv: f64,
    when: WhenType,
    guess: f64,
    tol: f64,
    maxiter: u32,
) -> Result<Option<f64>> {
    /*
       The rate of interest is computed by iteratively solving the (non-linear) equation:
       `fv + pv*(1+rate)**nper + pmt*(1+rate*when)/rate * ((1+rate)**nper - 1) = 0` for `rate`
    */
    // Assume all parameters are provided - deal with default arguments later

    let mut rn = guess;
    let mut iter: u32 = 0;
    let mut close = false;

    while iter < maxiter && !close {
        let rnp1 = rn - g_div_gp(rn, nper, pmt, pv, fv, when);
        let diff = (rnp1 - rn).abs();
        close = diff < tol;
        iter += 1;
        rn = rnp1;
    }

    // if convergence
    if close {
        debug!("Converged - {}, at: {}", rn, iter);
        Ok(Some(rn))
    // if no convergence after maxiter
    } else {
        debug!("Maximum iterations reached - {}, at: {}", maxiter, rn);
        Ok(None)
    }
}

/// Compute the interest rate from a hash map with keys of
/// (`nper`, `pmt`, `pv`, `fv`, `when`, `guess`, `tol`, `maxiter`)
/// Since [`HashMap`] requires values of same type, we need to wrap into a variant of enum
/// ## Example
/// ```rust
/// use rfinancial::*;
/// let mut map = ParaMap::new();
/// map.insert("nper".into(), ParaType::U32(10));
/// map.insert("pmt".into(), ParaType::F64(0.0));
/// map.insert("pv".into(), ParaType::F64(-3500.0));
/// map.insert("fv".into(), ParaType::F64(10000.0));
/// map.insert("when".into(), ParaType::When(WhenType::End));
/// map.insert("guess".into(), ParaType::F64(0.1));
/// map.insert("tol".into(), ParaType::F64(1e-6));
/// map.insert("maxiter".into(), ParaType::U32(100));
/// let result = rate_from_map(map).expect("Error computing rate");
/// println!("rate is {:#?}", result);
/// ```
pub fn rate_from_map(map: ParaMap) -> Result<Option<f64>> {
    let op = |err: Error| {
        Error::OtherError(format!(
            "Failed to compute `rate` from: `{:?}` <- {}",
            map, err
        ))
    };

    let nper = get_u32(&map, "nper").map_err(&op)?;
    let pmt = get_f64(&map, "pmt").map_err(&op)?;
    let pv = get_f64(&map, "pv").map_err(&op)?;
    let fv = get_f64(&map, "fv").map_err(&op)?;
    let when = get_when(&map, "when").map_err(&op)?;
    let guess = get_f64(&map, "guess").map_err(&op)?;
    let tol = get_f64(&map, "tol").map_err(&op)?;
    let maxiter = get_u32(&map, "maxiter").map_err(op)?;
    rate(nper, pmt, pv, fv, when, guess, tol, maxiter)
}

/// Builder for constructing `Rate` instances with a fluent API
///
/// Provides sensible defaults for optional parameters:
/// - `guess`: 0.1 (initial guess for rate)
/// - `tol`: 1e-6 (tolerance for convergence)
/// - `maxiter`: 100 (maximum iterations)
///
/// ## Example
/// ```rust
/// use rfinancial::*;
/// let rate = RateBuilder::new()
///     .nper(10)
///     .pv(-3500.0)
///     .fv(10000.0)
///     .build()?;
/// println!("rate is {:?}", rate.get());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone)]
pub struct RateBuilder {
    nper: u32,
    pmt: f64,
    pv: f64,
    fv: f64,
    when: WhenType,
    guess: f64,
    tol: f64,
    maxiter: u32,
}

impl RateBuilder {
    /// Create a new `RateBuilder` with default values
    pub fn new() -> Self {
        RateBuilder {
            nper: 1,
            pmt: 0.0,
            pv: 0.0,
            fv: 0.0,
            when: WhenType::End,
            guess: 0.1,
            tol: 1e-6,
            maxiter: 100,
        }
    }

    /// Set the number of compounding periods
    pub fn nper(mut self, nper: u32) -> Self {
        self.nper = nper;
        self
    }

    /// Set the payment in each period
    pub fn pmt(mut self, pmt: f64) -> Self {
        self.pmt = pmt;
        self
    }

    /// Set the present value
    pub fn pv(mut self, pv: f64) -> Self {
        self.pv = pv;
        self
    }

    /// Set the future value
    pub fn fv(mut self, fv: f64) -> Self {
        self.fv = fv;
        self
    }

    /// Set when payments are due (defaults to `WhenType::End`)
    pub fn when(mut self, when: WhenType) -> Self {
        self.when = when;
        self
    }

    /// Set the initial guess for rate solving (defaults to 0.1)
    pub fn guess(mut self, guess: f64) -> Self {
        self.guess = guess;
        self
    }

    /// Set the tolerance for convergence (defaults to 1e-6)
    pub fn tol(mut self, tol: f64) -> Self {
        self.tol = tol;
        self
    }

    /// Set the maximum number of iterations (defaults to 100)
    pub fn maxiter(mut self, maxiter: u32) -> Self {
        self.maxiter = maxiter;
        self
    }

    /// Build the `Rate` instance
    pub fn build(self) -> Result<Rate> {
        Ok(Rate {
            nper: self.nper,
            pmt: self.pmt,
            pv: self.pv,
            fv: self.fv,
            when: self.when,
            guess: self.guess,
            tol: self.tol,
            maxiter: self.maxiter,
        })
    }
}

impl Default for RateBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_rate_from_tuple() {
        let rate =
            Rate::from_tuple((10, 0.0, -3500.0, 10000.0, WhenType::End, 0.1, 1e-6, 100)).unwrap();
        // npf.rate(10, 0, -3500, 10000)
        // 0.11069085371426901
        let res = rate.get().unwrap().unwrap();
        let tgt = 0.11069085371426901;
        assert!(
            float_close(res, tgt, RTOL, ATOL),
            "{:#?} v.s. {:#?}",
            res,
            tgt
        );
    }

    #[test]
    fn test_rate_from_map() {
        let mut map = ParaMap::new();
        map.insert("nper".into(), ParaType::U32(10));
        map.insert("pmt".into(), ParaType::F64(0.0));
        map.insert("pv".into(), ParaType::F64(-3500.0));
        map.insert("fv".into(), ParaType::F64(10000.0));
        map.insert("when".into(), ParaType::When(WhenType::End));
        map.insert("guess".into(), ParaType::F64(0.1));
        map.insert("tol".into(), ParaType::F64(1e-6));
        map.insert("maxiter".into(), ParaType::U32(100));
        let rate = Rate::from_map(map).unwrap();
        // npf.rate(10, 0, -3500, 10000)
        // 0.11069085371426901
        let res = rate.get().unwrap().unwrap();
        let tgt = 0.11069085371426901;
        assert!(
            float_close(res, tgt, RTOL, ATOL),
            "{:#?} v.s. {:#?}",
            res,
            tgt
        );
    }

    #[test]
    fn test_rate_with_end() {
        let nper = 10;
        let pmt = 0.0;
        let pv = -3500.0;
        let fv = 10000.0;
        let when = WhenType::End;
        let guess = 0.1;
        let tol = 1e-6;
        let maxiter: u32 = 100;

        let rate = Rate {
            nper,
            pmt,
            pv,
            fv,
            when,
            guess,
            tol,
            maxiter,
        };

        // npf.rate(10, 0, -3500, 10000)
        // 0.11069085371426901
        let res = rate.get().unwrap().unwrap();
        let tgt = 0.11069085371426901;
        assert!(
            float_close(res, tgt, RTOL, ATOL),
            "{:#?} v.s. {:#?}",
            res,
            tgt
        );
    }

    #[test]
    fn test_rate_with_begin() {
        let nper = 10;
        let pmt = 0.0;
        let pv = -3500.0;
        let fv = 10000.0;
        let when = WhenType::Begin;
        let guess = 0.1;
        let tol = 1e-6;
        let maxiter: u32 = 100;

        let rate = Rate {
            nper,
            pmt,
            pv,
            fv,
            when,
            guess,
            tol,
            maxiter,
        };

        // npf.rate(10, 0, -3500, 10000, 'begin')
        // 0.11069085371426901
        let res = rate.get().unwrap().unwrap();
        let tgt = 0.11069085371426901;
        assert!(
            float_close(res, tgt, RTOL, ATOL),
            "{:#?} v.s. {:#?}",
            res,
            tgt
        );
    }

    #[test]
    fn test_rate_no_solution() {
        let nper = 12;
        let pmt = 400.0;
        let pv = 10000.0;
        let fv = 5000.0;
        let when = WhenType::End;
        let guess = 0.1;
        let tol = 1e-6;
        let maxiter: u32 = 100;

        let rate = Rate {
            nper,
            pmt,
            pv,
            fv,
            when,
            guess,
            tol,
            maxiter,
        };

        // npf.rate(12, 400, 10000, 5000)
        // nan
        let res = rate.get().unwrap();
        let tgt = None;
        assert_eq!(res, tgt, "{:#?} v.s. {:#?}", res, tgt);
    }

    #[test]
    fn test_rate_err() {
        let mut map = ParaMap::new();
        map.insert("Nper".into(), ParaType::U32(10));
        map.insert("pmt".into(), ParaType::F64(0.0));
        map.insert("pv".into(), ParaType::F64(-3500.0));
        map.insert("fv".into(), ParaType::F64(10000.0));
        map.insert("when".into(), ParaType::When(WhenType::End));
        map.insert("guess".into(), ParaType::F64(0.1));
        map.insert("tol".into(), ParaType::F64(1e-6));
        map.insert("maxiter".into(), ParaType::U32(100));
        let rate = Rate::from_map(map);
        assert!(rate.is_err());
    }

    #[test]
    fn test_rate_function() {
        // npf.rate(10, 0, -3500, 10000)
        // 0.11069085371426901
        let res = super::rate(10, 0.0, -3500.0, 10000.0, WhenType::End, 0.1, 1e-6, 100)
            .unwrap()
            .unwrap();
        let tgt = 0.11069085371426901;
        assert!(
            float_close(res, tgt, RTOL, ATOL),
            "{:#?} v.s. {:#?}",
            res,
            tgt
        );
    }

    #[test]
    fn test_rate_from_map_function() {
        let mut map = ParaMap::new();
        map.insert("nper".into(), ParaType::U32(10));
        map.insert("pmt".into(), ParaType::F64(0.0));
        map.insert("pv".into(), ParaType::F64(-3500.0));
        map.insert("fv".into(), ParaType::F64(10000.0));
        map.insert("when".into(), ParaType::When(WhenType::End));
        map.insert("guess".into(), ParaType::F64(0.1));
        map.insert("tol".into(), ParaType::F64(1e-6));
        map.insert("maxiter".into(), ParaType::U32(100));

        // npf.rate(10, 0, -3500, 10000)
        // 0.11069085371426901
        let res = super::rate_from_map(map).unwrap().unwrap();
        let tgt = 0.11069085371426901;
        assert!(
            float_close(res, tgt, RTOL, ATOL),
            "{:#?} v.s. {:#?}",
            res,
            tgt
        );
    }

    #[test]
    fn test_rate_builder() {
        let rate = RateBuilder::new()
            .nper(10)
            .pmt(0.0)
            .pv(-3500.0)
            .fv(10000.0)
            .when(WhenType::End)
            .guess(0.1)
            .tol(1e-6)
            .maxiter(100)
            .build()
            .unwrap();

        let res = rate.get().unwrap().unwrap();
        let tgt = 0.11069085371426901;
        assert!(
            float_close(res, tgt, RTOL, ATOL),
            "{:#?} v.s. {:#?}",
            res,
            tgt
        );
    }

    #[test]
    fn test_rate_builder_with_defaults() {
        // Test that builder uses sensible defaults
        let rate = RateBuilder::new()
            .nper(10)
            .pv(-3500.0)
            .fv(10000.0)
            .build()
            .unwrap();

        // Should produce the same result as with explicit defaults
        let res = rate.get().unwrap().unwrap();
        let tgt = 0.11069085371426901;
        assert!(
            float_close(res, tgt, RTOL, ATOL),
            "{:#?} v.s. {:#?}",
            res,
            tgt
        );
    }
}
