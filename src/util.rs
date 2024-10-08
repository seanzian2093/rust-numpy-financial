use crate::{Error, Result};
/// Tolerance of relative difference
pub const RTOL: f64 = 1e-10;
/// Tolerance of absolute difference
pub const ATOL: f64 = 1e-5;

/// To compare if two `f64` are close enough to be treated as `equal`
pub fn float_close(lhs: f64, rhs: f64, rtol: f64, atol: f64) -> bool {
    let cond1 = ((lhs - rhs) / rhs).abs() <= rtol;
    let cond2 = (lhs - rhs).abs() <= atol;

    cond1 | cond2
}

#[derive(Clone, Debug, PartialEq)]
/// when payments are due in a payment period
pub enum WhenType {
    End = 0,
    Begin = 1,
}

/// Parameter types in a enum
#[derive(Debug)]
pub enum ParaType {
    F64(f64),
    U32(u32),
    When(WhenType),
    VecF64(Vec<f64>),
}

pub type ParaMap = std::collections::HashMap<String, ParaType>;

pub fn get_f64(map: &ParaMap, field: &str) -> Result<f64> {
    if let Some(&ParaType::F64(v)) = map.get(field) {
        Ok(v)
    } else {
        Err(Error::ParaError(format!("{}: f64", field)))
    }
}

pub fn get_u32(map: &ParaMap, field: &str) -> Result<u32> {
    if let Some(&ParaType::U32(v)) = map.get(field) {
        Ok(v)
    } else {
        Err(Error::ParaError(format!("{}: u32", field)))
    }
}

pub fn get_when(map: &ParaMap, field: &str) -> Result<WhenType> {
    if let Some(&ParaType::When(ref v)) = map.get(field) {
        Ok(v.clone())
    } else {
        Err(Error::ParaError(format!("{}: WhenType", field)))
    }
}

pub fn get_vecf64(map: &ParaMap, field: &str) -> Result<Vec<f64>> {
    if let Some(&ParaType::VecF64(ref v)) = map.get(field) {
        Ok(v.clone())
    } else {
        Err(Error::ParaError(format!("{}: VecF64", field)))
    }
}
