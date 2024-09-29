use crate::{get_f64, get_vecf64, Error, ParaMap, Result};

/// # Compute the Modified Internal Rate of Return (MIRR)

/// MIRR is a financial metric that takes into account both the cost of the investment and the return on reinvested cash flows.
/// It is useful for evaluating the profitability of an investment with multiple cash inflows and outflows.
///
/// ## Parameters
/// * `values` : array_like. It must contain at least one positive and one negative value
/// * `finance_rate` : interest rate paid on the cash flows
/// * `reinvest_rate` : interest rate received on the cash flows upon reinvestment

/// ## Return:
/// * `mirr`: the modified internal rate of return

/// ## Example
/// ```rust
/// use rfinancial::*;
/// let tup = (vec![100.0, 200.0, -50.0, 300.00, -200.0], 0.05, 0.06);
/// let mirr = ModifiedIRR::from_tuple(tup);
/// println!("\n{:#?}'s mirr is {:#?}", mirr, mirr.get());
/// ```

#[derive(Debug)]
pub struct ModifiedIRR {
    values: Vec<f64>,
    finance_rate: f64,
    reinvest_rate: f64,
}

impl ModifiedIRR {
    /// Instantiate an instance of `ModifiedIRR` from a tuple of `(Vec<f64>, f64, f64>)` in said order
    pub fn from_tuple(tup: (Vec<f64>, f64, f64)) -> Self {
        ModifiedIRR {
            values: tup.0,
            finance_rate: tup.1,
            reinvest_rate: tup.2,
        }
    }

    /// Instantiate a `ModifiedIRR` instance from a hash map with keys of (`values`, `finance_rate`, `reinvest_rate`) in said order
    /// Since [`HashMap`] requires values of same type, we need to wrap into a variant of enum
    pub fn from_map(map: ParaMap) -> Result<Self> {
        let op = |err: Error| {
            Error::OtherError(format!(
                "Failed construct an instance of `ModifiedIRR` from: `{:?}` <- {}",
                map, err
            ))
        };

        let values = get_vecf64(&map, "values").map_err(|err| op(err))?;
        let finance_rate = get_f64(&map, "finance_rate").map_err(|err| op(err))?;
        let reinvest_rate = get_f64(&map, "reinvest_rate").map_err(|err| op(err))?;
        Ok(ModifiedIRR {
            values,
            finance_rate,
            reinvest_rate,
        })
    }

    fn mirr(&self) -> Result<Option<f64>> {
        let any_negative = self.values.iter().any(|&v| v <= 0.0);
        let any_positive = self.values.iter().any(|&v| v > 0.0);
        if !(any_negative & any_positive) {
            println!("No real solution exists for MIRR since  all cashflows are of the same sign.");
            Ok(None)
        } else {
            // v * neg
            let neg_pmts: Vec<f64> = self
                .values
                .iter()
                .map(|&rf| if rf < 0.0 { rf } else { 0.0 })
                .collect();

            // v * pos
            let pos_pmts: Vec<f64> = self
                .values
                .iter()
                .map(|&rf| if rf > 0.0 { rf } else { 0.0 })
                .collect();

            // numer = np.abs(npv(rr, v * pos))
            let numer = crate::NetPresentValue::from_tuple((pos_pmts, self.reinvest_rate))
                .get()?
                .abs();

            // denom = np.abs(npv(fr, v * neg))
            let denom = crate::NetPresentValue::from_tuple((neg_pmts, self.finance_rate))
                .get()?
                .abs();

            // (numer / denom) ** (1 / (n - 1)) * (1 + rr) - 1
            let n = self.values.len() as f64;
            let mirr = (numer / denom).powf(1.0 / (n - 1.0)) * (1.0 + self.reinvest_rate) - 1.0;
            Ok(Some(mirr))
        }
    }

    /// Get the `mirr` from an instance of `ModifiedIRR`
    pub fn get(&self) -> Result<Option<f64>> {
        self.mirr()
    }
}

#[allow(unused_imports)]
#[cfg(test)]
mod tests {
    use std::collections::btree_map::Values;

    use crate::*;

    #[test]
    fn test_mirr_from_tuple() {
        // case 1
        // npf.mirr([-120000, 39000, 30000, 21000, 37000, 46000], 0.10, 0.12)
        // 0.1260941303659051

        // let tup = (
        //     vec![-120000.0, 39000.0, 30000.0, 21000.0, 37000.0, 46000.0],
        //     0.10,
        //     0.12,
        // );
        // let tgt = 0.1260941303659051;

        // case 2
        // npf.mirr([100, 200, -50, 300, -200], 0.05, 0.06)
        // 0.3428233878421769;

        let tup = (vec![100.0, 200.0, -50.0, 300.00, -200.0], 0.05, 0.06);
        let tgt = 0.3428233878421769;

        let mirr = ModifiedIRR::from_tuple(tup);
        let res = mirr.get().unwrap().unwrap();
        assert!(
            float_close(res, tgt, RTOL, ATOL),
            "{:#?} v.s. {:#?}",
            res,
            tgt
        )
    }

    #[test]
    fn test_mirr_from_map() {
        // npf.mirr([100, 200, -50, 300, -200], 0.05, 0.06)
        // 0.3428233878421769;

        let tup = (vec![100.0, 200.0, -50.0, 300.00, -200.0], 0.05, 0.06);
        let mut map = ParaMap::new();
        map.insert("values".to_string(), ParaType::VecF64(tup.0));
        map.insert("finance_rate".to_string(), ParaType::F64(tup.1));
        map.insert("reinvest_rate".to_string(), ParaType::F64(tup.2));

        let tgt = 0.3428233878421769;

        let mirr = ModifiedIRR::from_map(map).unwrap();
        let res = mirr.get().unwrap().unwrap();
        assert!(
            float_close(res, tgt, RTOL, ATOL),
            "{:#?} v.s. {:#?}",
            res,
            tgt
        )
    }

    #[test]
    fn test_mirr_no_solution() {
        let mirr = ModifiedIRR::from_tuple((
            vec![39000.0, 30000.0, 21000.0, 37000.0, 46000.0],
            0.10,
            0.12,
        ));
        let res = mirr.get().unwrap();
        let tgt = None;
        assert_eq!(res, tgt, "{:#?} v.s. {:#?}", res, tgt)
    }

    #[test]
    fn test_mirr_err() {
        let tup = (vec![100.0, 200.0, -50.0, 300.00, -200.0], 0.05, 0.06);
        let mut map = ParaMap::new();
        map.insert("Values".to_string(), ParaType::VecF64(tup.0));
        map.insert("finance_rate".to_string(), ParaType::F64(tup.1));
        map.insert("reinvest_rate".to_string(), ParaType::F64(tup.2));

        let mirr = ModifiedIRR::from_map(map);
        let cond = mirr.is_err();
        assert!(cond);
    }

    #[test]
    #[ignore = "need to figure what is this case"]
    fn test_mirr_nan() {
        let tup = (
            vec![100.0, 200.0, -50.0, 300.00, -200.0],
            f64::MAX,
            f64::MAX,
        );
        let mut map = ParaMap::new();
        map.insert("values".to_string(), ParaType::VecF64(tup.0));
        map.insert("finance_rate".to_string(), ParaType::F64(tup.1));
        map.insert("reinvest_rate".to_string(), ParaType::F64(tup.2));

        let mirr = ModifiedIRR::from_map(map);
        let cond = mirr.unwrap().get().unwrap().unwrap().is_nan();
        assert!(cond);
    }
}
