use crate::{Error, FromMap, FromTuple, ParaMap, Result, get_f64, get_vecf64};

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
/// let mirr = ModifiedIRR::from_tuple(tup).expect("Error creating MIRR");
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
    ///
    /// # Errors
    /// Returns `ParaError` if:
    /// - `values` has fewer than 2 elements
    /// - `values` contains only positive or only negative values (MIRR requires both)
    pub fn from_tuple(tup: (Vec<f64>, f64, f64)) -> Result<Self> {
        let values = tup.0;

        if values.len() < 2 {
            return Err(Error::ParaError(
                "values must contain at least 2 elements".to_string(),
            ));
        }

        let has_positive = values.iter().any(|&v| v > 0.0);
        let has_negative = values.iter().any(|&v| v < 0.0);

        if !has_positive || !has_negative {
            return Err(Error::ParaError(
                "values must contain both positive and negative values for MIRR to exist"
                    .to_string(),
            ));
        }

        Ok(ModifiedIRR {
            values,
            finance_rate: tup.1,
            reinvest_rate: tup.2,
        })
    }

    /// Instantiate a `ModifiedIRR` instance from a hash map with keys of (`values`, `finance_rate`, `reinvest_rate`) in said order
    /// Since [`HashMap`] requires values of same type, we need to wrap into a variant of enum
    ///
    /// # Errors
    /// Returns error if map extraction or validation fails
    pub fn from_map(map: ParaMap) -> Result<Self> {
        let op = |err: Error| {
            Error::OtherError(format!(
                "Failed construct an instance of `ModifiedIRR` from: `{:?}` <- {}",
                map, err
            ))
        };

        let values = get_vecf64(&map, "values").map_err(&op)?;
        let finance_rate = get_f64(&map, "finance_rate").map_err(&op)?;
        let reinvest_rate = get_f64(&map, "reinvest_rate").map_err(op)?;

        Self::from_tuple((values, finance_rate, reinvest_rate))
            .map_err(|e| Error::OtherError(format!("MIRR validation failed: {}", e)))
    }

    fn mirr(&self) -> Result<Option<f64>> {
        // Validation is already done in from_tuple(), so we can assume:
        // - values.len() >= 2
        // - values contains both positive and negative values

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
        let npv_pos = crate::NetPresentValue::from_tuple((pos_pmts, self.reinvest_rate))?;
        let numer = npv_pos.get()?.abs();

        // denom = np.abs(npv(fr, v * neg))
        let npv_neg = crate::NetPresentValue::from_tuple((neg_pmts, self.finance_rate))?;
        let denom = npv_neg.get()?.abs();

        // (numer / denom) ** (1 / (n - 1)) * (1 + rr) - 1
        let n = self.values.len() as f64;
        let mirr = (numer / denom).powf(1.0 / (n - 1.0)) * (1.0 + self.reinvest_rate) - 1.0;
        Ok(Some(mirr))
    }

    /// Get the `mirr` from an instance of `ModifiedIRR`
    pub fn get(&self) -> Result<Option<f64>> {
        self.mirr()
    }
}

impl FromTuple<(Vec<f64>, f64, f64)> for ModifiedIRR {
    fn from_tuple(tup: (Vec<f64>, f64, f64)) -> Result<Self> {
        ModifiedIRR::from_tuple(tup)
    }
}

impl FromMap for ModifiedIRR {
    fn from_map(map: ParaMap) -> Result<Self> {
        ModifiedIRR::from_map(map)
    }
}

#[cfg(test)]
mod tests {

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

        let mirr = ModifiedIRR::from_tuple(tup).unwrap();
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
        // This test verifies that MIRR correctly fails when given values
        // with only positive cash flows (no negative values to discount)
        let result = ModifiedIRR::from_tuple((
            vec![39000.0, 30000.0, 21000.0, 37000.0, 46000.0],
            0.10,
            0.12,
        ));
        // Should return error because all values are positive
        assert!(result.is_err());
        // Verify the error message includes the key info
        if let Err(err) = result {
            let msg = err.to_string();
            assert!(msg.contains("positive and negative values"));
        }
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
