use crate::domain::provider::IFibonacciRiscZeroProvider;
use crate::prelude::FibchainError;
use alloy_sol_types::SolValue;
use methods::FIBONACCI_ELF;
use risc0_ethereum_contracts::{encode_seal, Seal};
use risc0_zkvm::{default_prover, ExecutorEnv, ProverOpts, VerifierContext};
use tracing::{error, info, instrument};

pub struct FibonacciRiscZeroProvider;

impl FibonacciRiscZeroProvider {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl IFibonacciRiscZeroProvider for FibonacciRiscZeroProvider {
    #[instrument(skip(self))]
    async fn generate_proof(&self, iterations: u16) -> crate::prelude::Result<(Vec<u8>, u128)> {
        info!(
            iterations = iterations,
            "Generating cryptographic proof of computation"
        );

        let input = iterations.abi_encode();
        let executor_environment =
            ExecutorEnv::builder()
                .write_slice(&input)
                .build()
                .map_err(|e| {
                    error!(
                        iterations = iterations,
                        input = hex::encode(&input),
                        "Failed to build executor environment: {}",
                        e
                    );

                    FibchainError::ZkVM(e.to_string())
                })?;

        info!(
            iterations = iterations,
            input = hex::encode(&input),
            "Proof generation started"
        );
        let receipt = default_prover()
            .prove_with_ctx(
                executor_environment,
                &VerifierContext::default(),
                FIBONACCI_ELF,
                &ProverOpts::groth16(),
            )
            .map_err(|e| {
                error!(
                    iterations = iterations,
                    input = hex::encode(&input),
                    "Failed to build proof: {}",
                    e
                );

                FibchainError::ZkVM(e.to_string())
            })?
            .receipt;

        info!(
            iterations = iterations,
            input = hex::encode(&receipt),
            "Proof generation finished. Extracting seal and journal..."
        );
        let seal = encode_seal(&receipt).map_err(|e| {
            error!(
                iterations = iterations,
                input = hex::encode(&input),
                journal = hex::encode(&receipt.journal.bytes),
                "Failed to encode seal: {}",
                e
            );

            FibchainError::ZkVM(e.to_string())
        })?;
        let journal = <u128>::abi_decode(&receipt.journal.bytes, true).map_err(|e| {
            error!(
                iterations = iterations,
                input = hex::encode(&input),
                journal = hex::encode(&receipt.journal.bytes),
                "Failed to decode journal: {}",
                e
            );

            FibchainError::ZkVM(e.to_string())
        })?;

        info!(
            iterations = iterations,
            input = hex::encode(&input),
            journal = hex::encode(&journal),
            "Proof generated"
        );
        Ok((seal, journal))
    }
}
