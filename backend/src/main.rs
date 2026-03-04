mod api;
mod chain;
mod rpc;
mod ws;

use std::sync::Arc;
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};

pub struct AppState {
    pub rpc: rpc::BitcoinRpc,
    pub chain: tokio::sync::Mutex<chain::ChainState>,
    pub tx: broadcast::Sender<ws::BlockEvent>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let rpc_url =
        std::env::var("BITCOIN_RPC_URL").unwrap_or_else(|_| "http://127.0.0.1:18443".into());
    let rpc_user = std::env::var("BITCOIN_RPC_USER").unwrap_or_else(|_| "bitcoin".into());
    let rpc_pass = std::env::var("BITCOIN_RPC_PASS").unwrap_or_else(|_| "bitcoin".into());

    let rpc = rpc::BitcoinRpc::new(&rpc_url, &rpc_user, &rpc_pass);
    let chain_state = chain::ChainState::new();
    let (tx, _rx) = broadcast::channel::<ws::BlockEvent>(100);

    let state = Arc::new(AppState {
        rpc,
        chain: tokio::sync::Mutex::new(chain_state),
        tx,
    });

    // Spawn the chain poller to detect new blocks
    let poll_state = Arc::clone(&state);
    tokio::spawn(async move {
        chain::poll_chain(poll_state).await;
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = api::router(Arc::clone(&state)).layer(cors);

    let addr = std::env::var("LISTEN_ADDR").unwrap_or_else(|_| "0.0.0.0:3001".into());
    tracing::info!("Orphan backend listening on {addr}");

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
