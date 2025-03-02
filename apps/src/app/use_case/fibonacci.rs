use crate::domain::provider::{IFibonacciEthereumProvider, IFibonacciRiscZeroProvider};
use crate::prelude::*;
use alloy_primitives::TxHash;
use std::pin::Pin;
use std::sync::Arc;
use tracing::{info, instrument};

#[derive(Clone)]
pub struct FibonacciGenerateNumberUseCase {
    fibonacci_risc_zero_provider: Arc<Pin<Box<dyn IFibonacciRiscZeroProvider + Sync + Send>>>,
    fibonacci_ethereum_provider: Arc<Pin<Box<dyn IFibonacciEthereumProvider + Sync + Send>>>,
}

impl FibonacciGenerateNumberUseCase {
    pub fn new(
        fibonacci_risc_zero_provider: Arc<Pin<Box<dyn IFibonacciRiscZeroProvider + Sync + Send>>>,
        fibonacci_ethereum_provider: Arc<Pin<Box<dyn IFibonacciEthereumProvider + Sync + Send>>>,
    ) -> Self {
        Self {
            fibonacci_risc_zero_provider,
            fibonacci_ethereum_provider,
        }
    }

    #[instrument(skip(self))]
    pub async fn execute(&self, iterations: u16) -> Result<TxHash> {
        info!("Executing Fibonacci number generation use-case");
        let (seal, fibonacci_number) = self
            .fibonacci_risc_zero_provider
            .generate_proof(iterations)
            .await?;

        self.fibonacci_ethereum_provider
            .increase_counter(fibonacci_number, seal)
            .await
    }
}
