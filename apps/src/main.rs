use std::str::FromStr;
use alloy::network::EthereumWallet;
use alloy::signers::local::PrivateKeySigner;
use std::sync::Arc;
use alloy_primitives::Address;

mod routes;
mod sol;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let private_key_hex =
        std::env::var("ETH_WALLET_PRIVATE_KEY").expect("ETH_WALLET_PRIVATE_KEY must be set");
    let private_key_hex = private_key_hex.split_at(2).1;
    let private_key = hex::decode(private_key_hex)?;
    
    let rpc_url = std::env::var("RPC_URL").expect("RPC_URL must be set");
    let rpc_url = rpc_url.parse()?;
    
    let contract_hex = std::env::var("ETH_CONTRACT").expect("ETH_CONTRACT must be set");
    let contract_address = Address::from_str(&contract_hex)?;

    let signer = PrivateKeySigner::from_slice(private_key.as_slice())?;
    let wallet = EthereumWallet::from(signer);

    let app_state = AppState {
        wallet: Arc::new(wallet),
        rpc_url,
        contract: contract_address
    };

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await?;
    let router = axum::Router::new()
        .merge(routes::Fibs::routes())
        .with_state(app_state);

    axum::serve(listener, router).await.map_err(|e| {
        log::error!("server error: {}", e);
        e.into()
    })
}

#[derive(Clone)]
pub struct AppState {
    pub wallet: Arc<EthereumWallet>,
    pub rpc_url: url::Url,
    pub contract: Address
}
