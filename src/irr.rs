use crate::util::{float_close, ATOL, RTOL};
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
/// ```rust
/// use rfinancial::*;
/// let values: Vec<f64> = vec![-150000.0, 15000.0, 25000.0, 35000.0, 45000.0, 60000.0];
/// let irr = InternalRateReturn::from_vec(values);
/// println!("{:#?}'s irr is {:#?}", irr, irr.get());
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
    pub fn from_vec(values: Vec<f64>) -> Self {
        // vec must at lease be of 2 elements
        // - to raise error in future not delegat to `irr`
        InternalRateReturn { values }
    }

    fn fx(v: &Vec<f64>, x: f64) -> f64 {
        let fx: f64 = v
            .iter()
            .rev()
            .enumerate()
            .map(|(p, c)| c * x.powf(p as f64))
            .sum();
        fx
    }
    fn dx(v: &Vec<f64>, x: f64) -> f64 {
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
        dx
    }

    // find 1st root
    fn find_root(v: &Vec<f64>) -> Option<f64> {
        // to re-implement
        let mut x = -0.9;
        let mut iter = 0;
        while iter < 100 {
            // f
            let f = Self::fx(v, x);
            // d
            let d = Self::dx(v, x);
            // if d is 0, update x and continue
            if float_close(d, 0.0, RTOL, ATOL) {
                x += 1.0;
                iter += 1;
                continue;
            };

            // x1
            let x1 = x - f / d;

            // if x and x1 are close enough return
            if float_close(x, x1, RTOL, ATOL) {
                return Some(x1);
            };

            // otherwise continue the loop - before next iteration, update x and iter
            x = x1;
            iter += 1;
        }
        // if maximum iteration reached, return roots or None
        None
    }

    // fina all possible roots- not used
    fn _find_roots(v: &Vec<f64>) -> Vec<f64> {
        // to re-implement
        let mut x = -10.0;
        let mut iter = 0;
        let mut roots = Vec::<f64>::new();
        while iter < 100 {
            // f
            let f = Self::fx(v, x);
            // d
            let d = Self::dx(v, x);
            // d is 0, update x and continue
            if float_close(d, 0.0, RTOL, ATOL) {
                x += 1.0;
                iter += 1;
                continue;
            };

            // x1
            let x1 = x - f / d;

            // if x and x1 are close enough return
            if float_close(x, x1, RTOL, ATOL) {
                roots.push(x1);
            };

            // otherwise continue the loop
            // update x and iter
            x = x1;
            iter += 1;
        }

        // if maximum iteration reached, return roots or None
        roots
    }

    fn irr(&self) -> Option<f64> {
        // vec must at lease be of 2 elements
        // - for now check at this function
        if self.values.len() <= 1 {
            return None;
        };
        // if signs of all elements of `values` are same, there is no solution
        let all_negative = self.values.iter().all(|&v| v <= 0.0);
        // - including all 0s
        let all_positive = self.values.iter().all(|&v| v > 0.0);
        if all_negative | all_positive {
            return None;
        };

        // Otherwise we are set to find irr

        // let g = Self::find_roots(&self.values);

        // - remove non-real ones
        // - f64 is real
        // - this filtering to be done in find roots step
        // let eirr: Vec<f64> = g.iter().map(|&v| v - 1.0).collect();

        // - remove those less than -1
        // let eirr: Vec<f64> = eirr.into_iter().filter(|&v| v >= -1.0).collect();

        // select one if ther are multiple
        // fn select_one(values: Vec<f64>) -> f64 {
        //     if values.len() == 1 {
        //         values[0]
        //     } else {
        //         values[0]
        //     }
        // }
        // Some(select_one(eirr))

        // For now use find_root, i.e. return one root or none
        let irr = Self::find_root(&self.values).unwrap() - 1.0;
        Some(irr)
    }

    /// Get the `irr` from an instance of `InternalRateReturn`
    pub fn get(&self) -> Option<f64> {
        self.irr()
    }
}

#[allow(unused_imports)]
mod test {
    use crate::*;

    #[test]
    fn test_fx() {
        let c: Vec<f64> = vec![1.0, 2.0, 3.0];
        let x = 2.0;
        let res = InternalRateReturn::fx(&c, x);
        // 1*x^2 + 2*x^1 + 3*x^0 ->
        // 1*2^2 + 2*2^1 + 3*2^0 -> 11
        let tgt = 11.0;
        assert_eq!(res, tgt, "{} v.s. {}", res, tgt);
    }

    #[test]
    fn test_dx() {
        let c: Vec<f64> = vec![1.0, 2.0, 3.0];
        let x = 2.0;
        let res = InternalRateReturn::dx(&c, x);
        // 1*x^2 + 2*x^1 + 3*x^0 ->
        // 1*2*x^1 + 2*1*x^0 + 0 ->
        // 1*2*2^1 + 2*1*2^0 + 0 ->
        let tgt = 6.0;
        assert_eq!(res, tgt, "{} v.s. {}", res, tgt);
    }

    #[test]
    fn test_find_root() {
        // -1.0 * x^2 + 1=0 -> x =1 and -1
        // let c: Vec<f64> = vec![-1.0, 0.0, 1.0];

        // - 0.25* x^2 + 1=0 -> x =2 and -2
        let c: Vec<f64> = vec![-0.25, 0.0, 1.0];

        let root = InternalRateReturn::find_root(&c).unwrap();
        let tgt = InternalRateReturn::fx(&c, root);
        let res = 0.0;
        assert!(
            float_close(res, tgt, RTOL, ATOL),
            "{:#?} v.s. {:#?}",
            res,
            tgt
        )
    }

    #[test]
    fn test_irr() {
        //         >>> npf.irr([-150000, 15000, 25000, 35000, 45000, 60000])
        // 0.052432888859413884
        let values: Vec<f64> = vec![-150000.0, 15000.0, 25000.0, 35000.0, 45000.0, 60000.0];
        let res = InternalRateReturn::from_vec(values).get().unwrap();
        let tgt = 0.052432888859413884;
        assert!(
            float_close(res, tgt, RTOL, ATOL),
            "{:#?} v.s. {:#?}",
            res,
            tgt
        )
    }
}
