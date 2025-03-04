#[derive(Debug, thiserror::Error)]
pub enum FibchainError {
    #[error("zkVM Error: {0}")]
    ZkVM(String),

    #[error(transparent)]
    Alloy(#[from] alloy::contract::Error),

    #[error(transparent)]
    AlloyPendingTransaction(#[from] alloy::providers::PendingTransactionError),

    #[error(transparent)]
    Generic(#[from] color_eyre::Report),
}
