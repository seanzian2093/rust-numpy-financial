// use core::fmt;

pub type Error = Box<dyn std::error::Error>;
// Customized Result
pub type Result<T> = std::result::Result<T, Error>;

// Customized Error
// Parameter Error

// #[derive(Debug)]
// pub struct ParaError;
// impl std::fmt::Display for ParaError {
//     fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> std::fmt::Result {
//         fmt.write_str(&format!(
//             "Error when getting parameter from the map:\n{self:?}"
//         ))
//     }
// }

// impl std::error::Error for ParaError {}
