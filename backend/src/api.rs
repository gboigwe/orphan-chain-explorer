use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde_json::{json, Value};
use std::sync::Arc;

use crate::ws;
use crate::AppState;

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/chain/info", get(chain_info))
        .route("/api/chain/tips", get(chain_tips))
        .route("/api/chain/blocks", get(chain_blocks))
        .route("/api/block/{hash}", get(block_info))
        .route("/api/mine", post(mine_block))
        .route("/api/mine/{hash}", post(mine_on_block))
        .route("/ws", get(ws::ws_handler))
        .with_state(state)
}

async fn health() -> Json<Value> {
    Json(json!({ "status": "ok", "service": "orphan-backend" }))
}

async fn chain_info(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let info = state
        .rpc
        .get_blockchain_info()
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, Json(json!({ "error": e }))))?;

    Ok(Json(serde_json::to_value(info).unwrap()))
}

async fn chain_tips(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let tips = state
        .rpc
        .get_chain_tips()
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, Json(json!({ "error": e }))))?;

    Ok(Json(json!({ "tips": tips })))
}

async fn chain_blocks(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // First trigger a resync to get latest state
    let tips = state.rpc.get_chain_tips().await.ok();
    if tips.is_some() {
        // Quick re-sync: update active status
        let best = state.rpc.get_best_block_hash().await.ok();
        let mut chain = state.chain.lock().await;
        if let Some(best_hash) = best {
            chain.best_hash = Some(best_hash);
        }
    }

    let chain = state.chain.lock().await;
    let blocks = chain.get_all_blocks();
    Ok(Json(json!({ "blocks": blocks })))
}

async fn block_info(
    State(state): State<Arc<AppState>>,
    Path(hash): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let block = state
        .rpc
        .get_block(&hash)
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, Json(json!({ "error": e }))))?;

    Ok(Json(serde_json::to_value(block).unwrap()))
}

/// Mine a block on the current best chain
async fn mine_block(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let address = state
        .rpc
        .get_new_address()
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, Json(json!({ "error": e }))))?;

    let hashes = state
        .rpc
        .generate_to_address(1, &address)
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, Json(json!({ "error": e }))))?;

    Ok(Json(json!({
        "mined": hashes,
        "address": address
    })))
}

/// Mine a block extending a specific block (for creating forks).
///
/// Strategy: Invalidate all active chain blocks from the target's height
/// upward, making the target block (or its ancestor) the tip. Then mine
/// one block. Finally reconsider invalidated blocks so both chains coexist.
async fn mine_on_block(
    State(state): State<Arc<AppState>>,
    Path(parent_hash): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let err = |e: String| (StatusCode::BAD_GATEWAY, Json(json!({ "error": e })));

    // Get info about the target block
    let target_block = state.rpc.get_block(&parent_hash).await.map_err(err)?;
    let target_height = target_block.height;
    let is_active = target_block.confirmations >= 0;

    // Get the current best tip
    let best_hash = state.rpc.get_best_block_hash().await.map_err(err)?;

    // If target is already the tip, just mine normally
    if parent_hash == best_hash {
        return mine_block(State(state)).await;
    }

    // Find the active chain block to invalidate.
    // For active blocks: invalidate the child (target_height + 1) so target becomes tip.
    // For stale blocks: invalidate the active block at target's height so Bitcoin Core
    //   falls back to the target's parent, then the target becomes the best fork.
    let invalidate_height = if is_active {
        target_height + 1
    } else {
        target_height
    };

    // Walk back from best tip to find the block at invalidate_height
    let mut blocks_to_invalidate = Vec::new();
    let mut h = best_hash.clone();
    loop {
        let b = state.rpc.get_block(&h).await.map_err(err)?;
        if b.height == invalidate_height {
            blocks_to_invalidate.push(h);
            break;
        }
        if b.height < invalidate_height {
            break;
        }
        match b.previousblockhash {
            Some(prev) => h = prev,
            None => break,
        }
    }

    // Invalidate to make the target (or its chain) the tip
    for hash in &blocks_to_invalidate {
        state.rpc.invalidate_block(hash).await.map_err(err)?;
    }

    // Now mine one block — Bitcoin Core will extend the current best tip
    let address = state.rpc.get_new_address().await.map_err(err)?;
    let mined = state
        .rpc
        .generate_to_address(1, &address)
        .await
        .map_err(err)?;

    // Reconsider the invalidated blocks so both chains coexist
    for hash in &blocks_to_invalidate {
        state.rpc.reconsider_block(hash).await.map_err(err)?;
    }

    // Broadcast chain update
    let _ = state.tx.send(ws::BlockEvent::ChainUpdate);

    Ok(Json(json!({
        "mined": mined,
        "parent": parent_hash,
        "address": address,
        "fork_created": !blocks_to_invalidate.is_empty()
    })))
}
