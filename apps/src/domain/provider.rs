use crate::prelude::*;
use alloy_primitives::TxHash;

#[async_trait::async_trait]
pub trait IFibonacciEthereumProvider {
    async fn increase_counter(&self, fibonacci_number: u128, seal: Vec<u8>) -> Result<TxHash>;
}

#[async_trait::async_trait]
pub trait IFibonacciRiscZeroProvider {
    async fn generate_proof(&self, iterations: u16) -> Result<(Vec<u8>, u128)>;
}
