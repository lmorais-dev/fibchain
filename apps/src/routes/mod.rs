use alloy::providers::ProviderBuilder;
use alloy_sol_types::SolValue;
use anyhow::Context;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use log::info;
use methods::FIBONACCI_ELF;
use risc0_ethereum_contracts::encode_seal;
use risc0_zkvm::{default_prover, ExecutorEnv, ProverOpts, VerifierContext};

pub struct Fibs;

#[derive(serde::Deserialize)]
struct GenerateAndPublishQueryParameters {
    pub iterations: u16,
}

impl Fibs {
    pub fn routes() -> Router<crate::AppState> {
        Router::new().route("/fibs", get(Self::generate_and_publish))
    }

    async fn generate_and_publish(
        State(state): State<crate::AppState>,
        Query(query): Query<GenerateAndPublishQueryParameters>,
    ) -> axum::response::Response {
        info!("Received request for query: {}", &query.iterations);
        let provider = ProviderBuilder::new()
            .with_recommended_fillers()
            .wallet(state.wallet.clone())
            .on_http(state.rpc_url.clone());

        let input = query.iterations.abi_encode();
        let executor_environment = ExecutorEnv::builder().write_slice(&input).build().unwrap();

        info!("Proving...");
        let receipt = default_prover()
            .prove_with_ctx(
                executor_environment,
                &VerifierContext::default(),
                FIBONACCI_ELF,
                &ProverOpts::groth16(),
            )
            .unwrap()
            .receipt;

        let seal = encode_seal(&receipt).unwrap();
        let journal = receipt.journal.bytes.clone();

        let fibonacci_num = <u128>::abi_decode(&journal, true)
            .context("decoding journal data")
            .unwrap();

        info!("Proving finished with result: {}", &fibonacci_num);
        info!("Sending proof to the blockchain...");
        let contract = crate::sol::IFibonacci::new(state.contract.clone(), provider);

        let inc_counter_call_builder = contract.increaseCounter(fibonacci_num, seal.into());
        let pending_tx = inc_counter_call_builder.send().await.unwrap();
        let tx = pending_tx.get_receipt().await.unwrap();
        
        info!("Sent with transaction id: {}", hex::encode(&tx.transaction_hash.0));
        (StatusCode::OK, Json(tx)).into_response()
    }
}
