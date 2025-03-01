use alloy::network::EthereumWallet;
use alloy::signers::local::PrivateKeySigner;
use alloy_primitives::Address;
use std::str::FromStr;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub wallet: Arc<EthereumWallet>,
    pub rpc_url: url::Url,
    pub contract: Address,
}

pub fn create_state() -> AppState {
    let private_key_hex =
        std::env::var("ETH_WALLET_PRIVATE_KEY").expect("ETH_WALLET_PRIVATE_KEY must be set");
    let private_key_hex = private_key_hex.split_at(2).1;
    let private_key = hex::decode(private_key_hex).expect("invalid private key");

    let rpc_url = std::env::var("RPC_URL").expect("RPC_URL must be set");
    let rpc_url = rpc_url.parse().expect("invalid RPC URL");

    let contract_hex = std::env::var("ETH_CONTRACT").expect("ETH_CONTRACT must be set");
    let contract_address = Address::from_str(&contract_hex).expect("invalid contract address");

    let signer = PrivateKeySigner::from_slice(private_key.as_slice()).expect("invalid private key");
    let wallet = EthereumWallet::from(signer);

    AppState {
        wallet: Arc::new(wallet),
        rpc_url,
        contract: contract_address,
    }
}
