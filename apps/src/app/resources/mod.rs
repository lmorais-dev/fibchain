use crate::prelude::{AppState, FibchainError};
use alloy::transports::http::reqwest::StatusCode;
use axum::{Json, Router};
use axum::response::IntoResponse;
use tracing::error;

pub mod fibonacci;

#[derive(Debug, Clone, serde::Serialize)]
struct ErrorMessageResponse {
    message: String,
}

pub trait Resource {
    fn routes() -> Router<AppState>;
}

pub fn fibchain_error_to_axum_response(error: &FibchainError) -> axum::response::Response {
    match error {
        FibchainError::ZkVM(message) => {
            error!(
                "Failed to generate fibonacci number due to a zkVM error: {}",
                message
            );

            let response = ErrorMessageResponse {
                message: message.to_owned(),
            };

            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
        FibchainError::Alloy(cause) => {
            error!(
                "Failed to generate fibonacci number due to an alloy error: {}",
                cause.to_string()
            );

            let response = ErrorMessageResponse {
                message: cause.to_string(),
            };

            (StatusCode::BAD_GATEWAY, Json(response)).into_response()
        }
        FibchainError::AlloyPendingTransaction(cause) => {
            error!(
                "Failed to generate fibonacci number due to an alloy error: {}",
                cause.to_string()
            );

            let response = ErrorMessageResponse {
                message: cause.to_string(),
            };

            (StatusCode::BAD_GATEWAY, Json(response)).into_response()
        }
    }
}
