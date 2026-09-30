use crate::{Error, FromMap, FromTuple, ParaMap, Result, get_f64, get_vecf64};

/// # Compute the net present value of a cash flow, given an interest rate
/// ## Parameters
/// * `rate` : an interest rate compounded once per period
/// * `values`: a cash flow, assume first payment is made at present, i.e. `t=0` the begining of 1st period
///
/// ## Return:
/// * `ipmt`: the net present value
///
/// ## Example
/// ```rust
/// use rfinancial::*;
/// let tup = (vec![-15000.0, 1500.0, 2500.0, 3500.0, 4500.0, 6000.0], 0.05);
/// let npv = NetPresentValue::from_tuple(tup).expect("Error creating NPV");
/// println!("{:#?}'s npv is {:?}", npv, npv.get());
/// ```
#[derive(Debug)]
pub struct NetPresentValue {
    values: Vec<f64>,
    rate: f64,
}

impl NetPresentValue {
    /// Instantiate a `NetPresentValue` instance from a vec of (`values`, `rate`) in said order
    ///
    /// # Errors
    /// Returns `ParaError` if `values` is empty
    pub fn from_tuple(tup: (Vec<f64>, f64)) -> Result<Self> {
        if tup.0.is_empty() {
            return Err(Error::ParaError("values must not be empty".to_string()));
        }
        Ok(NetPresentValue {
            values: tup.0,
            rate: tup.1,
        })
    }

    /// Instantiate a `NetPresentValue ` instance from a hash map with keys of (`values`, `rate`) in said order
    /// Since [`HashMap`] requires values of same type, we need to wrap into a variant of enum
    ///
    /// # Errors
    /// Returns error if map extraction or validation fails
    pub fn from_map(map: ParaMap) -> Result<Self> {
        let op = |err: Error| {
            Error::OtherError(format!(
                "Failed construct an instance of `NetPresentValue` from: `{:?}` <- {}",
                map, err
            ))
        };
        let values = get_vecf64(&map, "values").map_err(&op)?;
        let rate = get_f64(&map, "rate").map_err(op)?;
        Self::from_tuple((values, rate))
            .map_err(|e| Error::OtherError(format!("NPV validation failed: {}", e)))
    }

    fn npv(&self) -> Result<f64> {
        let npv: f64 = self
            .values
            .iter()
            .enumerate()
            .map(|(p, &c)| {
                let p = p as f64;
                c * (1.0 + self.rate).powf(-p)
            })
            .sum();

        Ok(npv)
    }

    pub fn get(&self) -> Result<f64> {
        self.npv()
    }
}

impl FromTuple<(Vec<f64>, f64)> for NetPresentValue {
    fn from_tuple(tup: (Vec<f64>, f64)) -> Result<Self> {
        NetPresentValue::from_tuple(tup)
    }
}

impl FromMap for NetPresentValue {
    fn from_map(map: ParaMap) -> Result<Self> {
        NetPresentValue::from_map(map)
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_npv_from_tuple() {
        // npf.npv(0.05, [-15000.0, 1500.0, 2500.0, 3500.0, 4500.0, 6000.0])
        // 122.89485495093959
        let tup = (vec![-15000.0, 1500.0, 2500.0, 3500.0, 4500.0, 6000.0], 0.05);
        let npv = NetPresentValue::from_tuple(tup).unwrap();
        let res = npv.get().unwrap();
        let tgt = 122.89485495093959;
        assert!(
            float_close(res, tgt, RTOL, ATOL),
            "{:#?} v.s. {:#?}",
            res,
            tgt
        );
    }

    #[test]
    fn test_npv_from_map() {
        // npf.npv(0.05, [-15000.0, 1500.0, 2500.0, 3500.0, 4500.0, 6000.0])
        // 122.89485495093959
        let values = vec![-15000.0, 1500.0, 2500.0, 3500.0, 4500.0, 6000.0];

        let mut map = ParaMap::new();
        map.insert("values".to_string(), ParaType::VecF64(values));
        map.insert("rate".to_string(), ParaType::F64(0.05));

        let npv = NetPresentValue::from_map(map);
        let res = npv.unwrap().get().unwrap();
        let tgt = 122.89485495093959;
        assert!(
            float_close(res, tgt, RTOL, ATOL),
            "{:#?} v.s. {:#?}",
            res,
            tgt
        );
    }

    #[test]
    fn test_npv_zero_rate() {
        // npf.npv(0.05, [-15000.0, 1500.0, 2500.0, 3500.0, 4500.0, 6000.0])
        // 122.89485495093959
        let tup = (vec![-15000.0, 1500.0, 2500.0, 3500.0, 4500.0, 6000.0], 0.0);
        let npv = NetPresentValue::from_tuple(tup).unwrap();
        let res = npv.get().unwrap();
        let tgt = 3000.0;
        assert!(
            float_close(res, tgt, RTOL, ATOL),
            "{:#?} v.s. {:#?}",
            res,
            tgt
        );
    }

    #[test]
    fn test_npv_err() {
        let values = vec![-15000.0, 1500.0, 2500.0, 3500.0, 4500.0, 6000.0];

        let mut map = ParaMap::new();
        map.insert("Values".to_string(), ParaType::VecF64(values));
        map.insert("rate".to_string(), ParaType::F64(0.05));

        let npv = NetPresentValue::from_map(map);
        let cond = npv.is_err();
        assert!(cond);
    }
}
