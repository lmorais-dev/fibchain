pub use crate::infra::app_state::*;
pub use crate::infra::error::*;
pub use crate::infra::sol::*;

pub type Result<T> = color_eyre::Result<T, FibchainError>;
pub type AxumResult<T> = color_eyre::Result<T, axum::http::StatusCode>;
