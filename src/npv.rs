use crate::{get_f64, get_vecf64, ParaMap};

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
/// let npv = NetPresentValue::from_tuple(tup);
/// println!("{:#?}'s npv is {:?}", npv, npv.get());
/// ```

#[derive(Debug)]
pub struct NetPresentValue {
    values: Vec<f64>,
    rate: f64,
}

impl NetPresentValue {
    /// Instantiate a `ModifiedIRR` instance from a vec of (`values`, `rate`) in said order
    pub fn from_tuple(tup: (Vec<f64>, f64)) -> Self {
        NetPresentValue {
            values: tup.0,
            rate: tup.1,
        }
    }

    /// Instantiate a `NetPresentValue ` instance from a hash map with keys of (`values`, `rate`) in said order
    /// Since [`HashMap`] requires values of same type, we need to wrap into a variant of enum
    pub fn from_map(map: ParaMap) -> Self {
        let values = get_vecf64(&map, "values").unwrap();
        let rate = get_f64(&map, "rate").unwrap();
        NetPresentValue { values, rate }
    }

    fn npv(&self) -> f64 {
        let npv: f64 = self
            .values
            .iter()
            .enumerate()
            .map(|(p, &c)| {
                let p = p as f64;
                c * (1.0 + self.rate).powf(-p)
            })
            .sum();

        npv
    }

    pub fn get(&self) -> f64 {
        self.npv()
    }
}

#[allow(unused_imports)]
#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_npv_from_tuple() {
        // npf.npv(0.05, [-15000.0, 1500.0, 2500.0, 3500.0, 4500.0, 6000.0])
        // 122.89485495093959
        let tup = (vec![-15000.0, 1500.0, 2500.0, 3500.0, 4500.0, 6000.0], 0.05);
        let npv = NetPresentValue::from_tuple(tup);
        let res = npv.get();
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
        let res = npv.get();
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
        let npv = NetPresentValue::from_tuple(tup);
        let res = npv.get();
        let tgt = 3000.0;
        assert!(
            float_close(res, tgt, RTOL, ATOL),
            "{:#?} v.s. {:#?}",
            res,
            tgt
        );
    }
}
