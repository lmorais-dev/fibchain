#[derive(Debug, thiserror::Error)]
pub enum FibchainError {
    #[error("zkVM Error: {0}")]
    ZkVM(String),

    #[error(transparent)]
    StdIo(std::io::Error),
}
