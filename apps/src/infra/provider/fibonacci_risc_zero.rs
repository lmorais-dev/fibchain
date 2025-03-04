use crate::domain::provider::IFibonacciRiscZeroProvider;
use crate::prelude::FibchainError;
use alloy_sol_types::SolValue;
use methods::FIBONACCI_ELF;
use risc0_ethereum_contracts::encode_seal;
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
        if iterations == 0 {
            error!(iterations = iterations, "Iterations cannot be zero!");
            return Err(FibchainError::Generic(color_eyre::eyre::eyre!(
                "Iterations cannot be zero!"
            )));
        }
        
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
            input = hex::encode(&input),
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
            journal = hex::encode(&journal.to_ne_bytes()),
            "Proof generated"
        );
        Ok((seal, journal))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::provider::IFibonacciRiscZeroProvider;
    use color_eyre::Result;
    use tracing_subscriber;

    #[tokio::test]
    async fn test_generate_proof_success() -> Result<()> {
        tracing_subscriber::fmt()
            .with_env_filter("info")
            .try_init()
            .ok();

        let provider = FibonacciRiscZeroProvider::new();
        let iterations: u16 = 5;

        let result = provider.generate_proof(iterations).await;

        assert!(
            result.is_ok(),
            "Expected success, but got an error: {:?}",
            result
        );
        let (seal, journal) = result?;

        assert!(!seal.is_empty(), "Seal should not be empty");
        assert!(journal > 0, "Journal should be a positive value");

        Ok(())
    }

    #[tokio::test]
    async fn test_generate_proof_invalid_iterations() -> Result<()> {
        tracing_subscriber::fmt()
            .with_env_filter("info")
            .try_init()
            .ok();

        let provider = FibonacciRiscZeroProvider::new();
        let iterations: u16 = 0; // Use an invalid edge case value

        let result = provider.generate_proof(iterations).await;

        assert!(result.is_err(), "Expected an error, but got success");
        let error = result.unwrap_err();
        assert!(
            format!("{:?}", error).contains("zero"),
            "Unexpected error message: {:?}",
            error
        );

        Ok(())
    }
}
