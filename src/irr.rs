use crate::{ATOL, Error, FromMap, FromTuple, ParaMap, RTOL, Result, float_close, get_vecf64};
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

    fn fx(v: &[f64], x: f64) -> Result<f64> {
        let fx: f64 = v
            .iter()
            .rev()
            .enumerate()
            .map(|(p, c)| c * x.powf(p as f64))
            .sum();
        Ok(fx)
    }

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

    // find 1st root
    fn find_root(v: &[f64]) -> Result<Option<f64>> {
        // to re-implement
        let mut x = -0.9;
        let mut iter = 0;
        while iter < 100 {
            // f
            let f = Self::fx(v, x)?;
            // d
            let d = Self::dx(v, x)?;
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
                return Ok(Some(x1));
            };

            // otherwise continue the loop - before next iteration, update x and iter
            x = x1;
            iter += 1;
        }
        // if maximum iteration reached, return roots or None
        Ok(None)
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
/// ## Example
/// ```rust
/// use rfinancial::*;
/// let values: Vec<f64> = vec![-150000.0, 15000.0, 25000.0, 35000.0, 45000.0, 60000.0];
/// let result = irr(&values).expect("Error computing irr");
/// println!("irr is {:?}", result);
/// ```
pub fn irr(values: &[f64]) -> Result<Option<f64>> {
    validate_values(values)?;
    let irr = InternalRateReturn::find_root(values)?.unwrap() - 1.0;
    Ok(Some(irr))
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
