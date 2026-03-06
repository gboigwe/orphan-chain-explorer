use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::IntoResponse,
};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum BlockEvent {
    #[serde(rename = "new_block")]
    NewBlock { hash: String },
    #[serde(rename = "reorg")]
    Reorg {
        old_tip: String,
        new_tip: String,
        depth: u64,
    },
    #[serde(rename = "chain_update")]
    ChainUpdate,
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.tx.subscribe();

    // Send the current chain state immediately on connect
    {
        let chain = state.chain.lock().await;
        let blocks = chain.get_all_blocks();
        if let Ok(json) = serde_json::to_string(&serde_json::json!({
            "type": "initial_state",
            "blocks": blocks
        })) {
            let _ = sender.send(Message::Text(json.into())).await;
        }
    }

    // Forward broadcast events to this WebSocket client
    let mut send_task = tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            if let Ok(json) = serde_json::to_string(&event) {
                if sender.send(Message::Text(json.into())).await.is_err() {
                    break;
                }
            }
        }
    });

    // Handle incoming messages (keep-alive pongs, etc.)
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Close(_) => break,
                _ => {} // ignore other messages for now
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    }

    tracing::info!("WebSocket client disconnected");
}
