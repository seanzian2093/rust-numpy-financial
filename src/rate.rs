use crate::{get_f64, get_u32, get_when, util::WhenType, ParaMap};
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
/// ```rust
/// use rfinancial::*;
/// let rate = Rate::from_tuple((10, 0.0, -3500.0, 10000.0, WhenType::End, 0.1, 1e-6, 100));
/// println!("{:#?}'s rate is {:#?}", rate, rate.get());
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
    pub fn from_tuple(tup: (u32, f64, f64, f64, WhenType, f64, f64, u32)) -> Self {
        Rate {
            nper: tup.0,
            pmt: tup.1,
            pv: tup.2,
            fv: tup.3,
            when: tup.4,
            guess: tup.5,
            tol: tup.6,
            maxiter: tup.7,
        }
    }

    /// Instantiate a `Rate` instance from a hash map with keys of (`nper`, `pmt`, `pv`, `fv`, `when`, `guess`, `tol`, `maxiter`) in said order
    /// Since [`HashMap`] requires values of same type, we need to wrap into a variant of enum
    pub fn from_map(map: ParaMap) -> Self {
        let nper = get_u32(&map, "nper").unwrap();
        let pmt = get_f64(&map, "pmt").unwrap();
        let pv = get_f64(&map, "pv").unwrap();
        let fv = get_f64(&map, "fv").unwrap();
        let when = get_when(&map, "when").unwrap();
        let guess = get_f64(&map, "guess").unwrap();
        let tol = get_f64(&map, "tol").unwrap();
        let maxiter = get_u32(&map, "maxiter").unwrap();
        Rate {
            nper,
            pmt,
            pv,
            fv,
            when,
            guess,
            tol,
            maxiter,
        }
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

    fn rate(&self) -> Option<f64> {
        /*
           The rate of interest is computed by iteratively solving the (non-linear) equation:
           `fv + pv*(1+rate)**nper + pmt*(1+rate*when)/rate * ((1+rate)**nper - 1) = 0` for `rate`
        */
        // Assume all parameters are provided - deal with default arguments later

        // rn = guess
        // iterator = 0
        // close = False
        // while (iterator < maxiter) and not np.all(close):
        //     rnp1 = rn - _g_div_gp(rn, nper, pmt, pv, fv, when)
        //     diff = abs(rnp1 - rn)
        //     close = diff < tol
        //     iterator += 1
        //     rn = rnp1

        let mut rn = self.guess;
        let mut iter: u32 = 0;
        let mut close = false;

        while (iter < self.maxiter) & (!close) {
            let rnp1 =
                rn - Self::_g_div_gp(rn, self.nper, self.pmt, self.pv, self.fv, self.when.clone());
            let diff = (rnp1 - rn).abs();
            close = diff < self.tol;
            iter += 1;
            rn = rnp1;
        }

        // if convergence
        if close {
            println!("Converged - {}, at: {}", rn, iter);
            return Some(rn);
        // if no convergence after maxiter
        } else {
            println!("Maximum iterations reached - {}, at: {}", self.maxiter, rn);
            return None;
        }
    }

    /// Get the rate from an instance of `Rate`
    pub fn get(&self) -> Option<f64> {
        self.rate()
    }
}

#[allow(unused_imports)]
#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_rate_from_tuple() {
        let rate = Rate::from_tuple((10, 0.0, -3500.0, 10000.0, WhenType::End, 0.1, 1e-6, 100));
        // npf.rate(10, 0, -3500, 10000)
        // 0.11069085371426901
        let res = rate.get().unwrap();
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
        let rate = Rate::from_map(map);
        // npf.rate(10, 0, -3500, 10000)
        // 0.11069085371426901
        let res = rate.get().unwrap();
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
        let res = rate.get().unwrap();
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
        let res = rate.get().unwrap();
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
        let res = rate.get();
        let tgt = None;
        assert_eq!(res, tgt, "{:#?} v.s. {:#?}", res, tgt);
    }
}
