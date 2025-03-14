use crate::app::resources::fibonacci::FibonacciResource;
use crate::app::resources::Resource;
use infra::{app_state, observability};

mod app;
mod domain;
mod infra;
mod prelude;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    color_eyre::install().ok();
    let _guard = observability::setup_tracing();

    let state = app_state::create_state();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await?;
    let router = axum::Router::new()
        .nest("/fibonacci", FibonacciResource::routes())
        .with_state(state);

    axum::serve(listener, router).await.map_err(|e| {
        tracing::error!("server error: {}", e);
        e.into()
    })
}
