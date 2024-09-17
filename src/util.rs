pub const RTOL: f64 = 1e-10;
pub const ATOL: f64 = 1e-5;
pub fn float_close(lhs: f64, rhs: f64, rtol: f64, atol: f64) -> bool {
    let cond1 = ((lhs - rhs) / rhs).abs() <= rtol;
    let cond2 = (lhs - rhs).abs() <= atol;

    cond1 | cond2
}

#[derive(Clone, Debug)]
pub enum WhenType {
    End = 0,
    Begin = 1,
}
