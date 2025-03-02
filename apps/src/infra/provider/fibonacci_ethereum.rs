use crate::domain::provider::IFibonacciEthereumProvider;
use crate::prelude::{FibchainError, IFibonacci};
use alloy::network::EthereumWallet;
use alloy::providers::ProviderBuilder;
use alloy_primitives::{Address, TxHash};
use risc0_ethereum_contracts::Seal;
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
        seal: &Seal,
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
                    seal = hex::encode(&seal),
                    journal = hex::encode(&fibonacci_number),
                    "Failed to send the proof to the contract: {}",
                    e
                );

                FibchainError::Alloy(e)
            })?
            .with_timeout(Some(Duration::from_secs(BLOCKCHAIN_TX_TIMEOUT_SECS as u64)))
            .with_required_confirmations(BLOCKCHAIN_TX_CONFIRMATIONS as u64);

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
                transaction_hash = hex::encode(&pending_transaction.tx_hash().0),
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
