use crate::prelude::*;
use alloy_primitives::TxHash;
use risc0_ethereum_contracts::Seal;

#[async_trait::async_trait]
pub trait IFibonacciEthereumProvider {
    async fn increase_counter(&self, fibonacci_number: u128, seal: &Seal) -> Result<TxHash>;
}

#[async_trait::async_trait]
pub trait IFibonacciRiscZeroProvider {
    async fn generate_proof(&self, iterations: u16) -> Result<(Vec<u8>, u128)>;
}
