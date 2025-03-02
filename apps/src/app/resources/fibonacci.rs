use crate::app::resources::{fibchain_error_to_axum_response, Resource};
use crate::prelude::*;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use tracing::instrument;

#[derive(Clone)]
pub struct FibonacciResource;

#[derive(Debug, Clone, Copy, serde::Deserialize)]
struct GenerateAndPublishQueryParameters {
    pub iterations: u16,
}

#[derive(Debug, Clone, serde::Serialize)]
struct GenerateNumberResponse {
    transaction_hash: String,
}

impl Resource for FibonacciResource {
    fn routes() -> Router<AppState> {
        Router::new().route("/", get(Self::generate_number))
    }
}

impl FibonacciResource {
    #[instrument(skip(state))]
    async fn generate_number(
        State(state): State<AppState>,
        Query(query): Query<GenerateAndPublishQueryParameters>,
    ) -> AxumResult<axum::response::Response> {
        let generation_result = state
            .fibonacci_number_generator
            .execute(query.iterations)
            .await;

        match generation_result {
            Ok(transaction_hash) => {
                let transaction_hash = hex::encode(transaction_hash);
                let response = GenerateNumberResponse { transaction_hash };
                Ok((StatusCode::OK, Json(response)).into())
            }
            Err(error) => Ok(fibchain_error_to_axum_response(&error)),
        }
    }
}
