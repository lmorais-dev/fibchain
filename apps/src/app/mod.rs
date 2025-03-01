use alloy::network::ReceiptResponse;
use alloy::providers::ProviderBuilder;
use alloy::rpc::types::TransactionReceipt;
use alloy_sol_types::SolValue;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use methods::FIBONACCI_ELF;
use risc0_ethereum_contracts::encode_seal;
use risc0_zkvm::{default_prover, ExecutorEnv, ProverOpts, VerifierContext};
use std::time::Duration;
use tracing::{error, info, instrument};

use crate::prelude::*;

static BLOCKCHAIN_TX_CONFIRMATIONS: u8 = 10;
static BLOCKCHAIN_TX_TIMEOUT_SECS: u16 = 60;

pub struct Fibs;

#[derive(Debug, Clone, Copy, serde::Deserialize)]
struct GenerateAndPublishQueryParameters {
    pub iterations: u16,
}

impl Fibs {
    pub fn routes() -> Router<AppState> {
        Router::new().route("/fibs", get(Self::generate_and_publish))
    }

    #[instrument(skip(state))]
    async fn generate_and_publish(
        State(state): State<AppState>,
        Query(query): Query<GenerateAndPublishQueryParameters>,
    ) -> AxumResult<Json<TransactionReceipt>> {
        let input = query.iterations.abi_encode();
        let executor_environment =
            ExecutorEnv::builder()
                .write_slice(&input)
                .build()
                .map_err(|e| {
                    error!("Failed to build an executor environment: {}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;

        info!("Proof generation started");
        let receipt = default_prover()
            .prove_with_ctx(
                executor_environment,
                &VerifierContext::default(),
                FIBONACCI_ELF,
                &ProverOpts::groth16(),
            )
            .map_err(|e| {
                error!("Failed to generate proof: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?
            .receipt;

        let seal = encode_seal(&receipt).map_err(|e| {
            error!(
                seal_size = receipt.seal_size(),
                journal = hex::encode(receipt.journal.bytes.clone()),
                "Failed to encode seal: {}",
                e
            );
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        let journal = receipt.journal.bytes.clone();
        let fibonacci_num = <u128>::abi_decode(&journal, true).map_err(|e| {
            error!(
                journal = hex::encode(journal),
                "Failed to decode journal: {}", e
            );
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        info!("Proving finished with result: {}", &fibonacci_num);

        info!("Sending proof to the blockchain...");
        let provider = ProviderBuilder::new()
            .with_recommended_fillers()
            .wallet(state.wallet.clone())
            .on_http(state.rpc_url.clone());
        let contract = IFibonacci::new(state.contract.clone(), provider);
        let inc_counter_call_builder = contract.increaseCounter(fibonacci_num, seal.clone().into());
        let pending_tx = inc_counter_call_builder
            .send()
            .await
            .map_err(|e| {
                error!(
                    contract = hex::encode(&contract.address().0),
                    seal = hex::encode(&seal),
                    journal = hex::encode(&receipt.journal.bytes),
                    "Failed to invoke contract counter: {}",
                    e
                );

                StatusCode::BAD_GATEWAY
            })?
            .with_timeout(Some(Duration::from_secs(BLOCKCHAIN_TX_TIMEOUT_SECS as u64)))
            .with_required_confirmations(BLOCKCHAIN_TX_CONFIRMATIONS as u64);

        info!(
            transaction_hash = hex::encode(pending_tx.tx_hash().0),
            timeout = BLOCKCHAIN_TX_TIMEOUT_SECS,
            confirmations = BLOCKCHAIN_TX_CONFIRMATIONS,
            contract = hex::encode(&contract.address().0),
            "Waiting for the transaction to get confirmed..."
        );
        let tx = pending_tx.get_receipt().await.map_err(|e| {
            error!("Failed to get pending tx: {}", e);
            StatusCode::BAD_GATEWAY
        })?;

        info!(
            block_hash = hex::encode(&tx.block_hash().unwrap_or_default().0),
            transaction_hash = hex::encode(&tx.transaction_hash.0),
            contract = hex::encode(&contract.address().0),
            "Transaction Confirmed. Success!"
        );
        Ok(Json(tx))
    }
}
