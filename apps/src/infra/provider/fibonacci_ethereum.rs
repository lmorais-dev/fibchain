use crate::domain::provider::IFibonacciEthereumProvider;
use crate::prelude::{FibchainError, IFibonacci};
use alloy::network::EthereumWallet;
use alloy::providers::ProviderBuilder;
use alloy_primitives::{Address, TxHash};
use alloy_sol_types::SolValue;
use std::time::Duration;
use tracing::{error, info, instrument};

static BLOCKCHAIN_TX_CONFIRMATIONS: u8 = 10;
static BLOCKCHAIN_TX_TIMEOUT_SECS: u16 = 60;

#[derive(Clone)]
pub struct FibonacciEthereumProvider {
    wallet: EthereumWallet,
    contract: Address,
    rpc_url: url::Url,
}

impl FibonacciEthereumProvider {
    pub fn new(wallet: EthereumWallet, contract: Address, rpc_url: url::Url) -> Self {
        Self {
            wallet,
            contract,
            rpc_url,
        }
    }
}

#[async_trait::async_trait]
impl IFibonacciEthereumProvider for FibonacciEthereumProvider {
    #[instrument(skip(self, seal))]
    async fn increase_counter(
        &self,
        fibonacci_number: u128,
        seal: Vec<u8>,
    ) -> crate::prelude::Result<TxHash> {
        info!("Sending cryptographic proof to the contract");
        let fill_provider = ProviderBuilder::new()
            .with_recommended_fillers()
            .wallet(self.wallet.clone())
            .on_http(self.rpc_url.clone());
        let contract = IFibonacci::new(self.contract.clone(), fill_provider);
        let call_builder = contract.increaseCounter(fibonacci_number, seal.clone().into());
        let pending_transaction = call_builder
            .send()
            .await
            .map_err(|e| {
                error!(
                    contract = hex::encode(&contract.address().0),
                    seal = hex::encode(&seal.abi_encode()),
                    journal = hex::encode(&fibonacci_number.to_ne_bytes()),
                    "Failed to send the proof to the contract: {}",
                    e
                );

                FibchainError::Alloy(e)
            })?
            .with_timeout(Some(Duration::from_secs(BLOCKCHAIN_TX_TIMEOUT_SECS as u64)))
            .with_required_confirmations(BLOCKCHAIN_TX_CONFIRMATIONS as u64);

        let transaction_hash = pending_transaction.tx_hash().clone();

        info!(
            contract = hex::encode(&contract.address().0),
            timeout = BLOCKCHAIN_TX_TIMEOUT_SECS as u64,
            confirmations = BLOCKCHAIN_TX_CONFIRMATIONS as u64,
            transaction_hash = hex::encode(&pending_transaction.tx_hash().0),
            "Sent! Waiting for confirmation..."
        );
        let transaction = pending_transaction.get_receipt().await.map_err(|e| {
            error!(
                contract = hex::encode(&contract.address().0),
                transaction_hash = hex::encode(transaction_hash.0),
                "Error while waiting for confirmations: {}",
                e
            );
            FibchainError::AlloyPendingTransaction(e)
        })?;

        info!(
            contract = hex::encode(&contract.address().0),
            transaction_hash = hex::encode(&transaction.transaction_hash.0),
            block_hash = hex::encode(&transaction.block_hash.unwrap_or_default().0),
            "Transaction Confirmed. Success!"
        );
        Ok(transaction.transaction_hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::provider::{IFibonacciEthereumProvider, IFibonacciRiscZeroProvider};
    use alloy::network::EthereumWallet;
    use alloy::signers::local::PrivateKeySigner;
    use alloy_primitives::Address;
    use std::str::FromStr;
    use url::Url;
    use crate::infra::provider::fibonacci_risc_zero::FibonacciRiscZeroProvider;

    #[tokio::test]
    async fn test_increase_counter_success() {
        let wallet = EthereumWallet::new(
            PrivateKeySigner::from_str(
                "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80",
            )
            .unwrap(),
        );
        let contract = Address::from_str("0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512").unwrap();
        let rpc_url = Url::parse("http://localhost:8545").unwrap();

        let risc_zero_provider = FibonacciRiscZeroProvider::new();
        let provider = FibonacciEthereumProvider::new(wallet, contract, rpc_url);

        // Generate a proof
        let (seal, num) = risc_zero_provider.generate_proof(5).await.unwrap();
        
        // Call increase_counter
        let result = provider.increase_counter(num, seal).await;

        // Assert that result is Ok
        assert!(result.is_ok(), "Expected Ok result, got: {:?}", result);
    }

    #[tokio::test]
    async fn test_increase_counter_error_handling() {
        let wallet = EthereumWallet::new(
            PrivateKeySigner::from_str(
                "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80",
            )
            .unwrap(),
        );
        let contract = Address::from_str("0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512").unwrap();
        let rpc_url = Url::parse("http://localhost:8545").unwrap();

        let provider = FibonacciEthereumProvider::new(wallet, contract, rpc_url);

        // Simulate an invalid seal or connection issue
        let result = provider.increase_counter(21, vec![]).await;

        // Assert that result is Err
        assert!(result.is_err(), "Expected Err result, got: {:?}", result);
    }
}
