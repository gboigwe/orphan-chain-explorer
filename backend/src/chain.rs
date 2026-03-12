use crate::rpc::BlockInfo;
use crate::ws::BlockEvent;
use crate::AppState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// A simplified block for the tree representation sent to the frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockNode {
    pub hash: String,
    pub height: u64,
    pub prev_hash: Option<String>,
    pub time: u64,
    pub n_tx: u64,
    pub size: u64,
    pub weight: u64,
    pub is_active: bool,
    pub confirmations: i64,
}

impl From<&BlockInfo> for BlockNode {
    fn from(b: &BlockInfo) -> Self {
        Self {
            hash: b.hash.clone(),
            height: b.height,
            prev_hash: b.previousblockhash.clone(),
            time: b.time,
            n_tx: b.n_tx,
            size: b.size,
            weight: b.weight,
            is_active: b.confirmations >= 0,
            confirmations: b.confirmations,
        }
    }
}

pub struct ChainState {
    /// All known blocks indexed by hash
    pub blocks: HashMap<String, BlockNode>,
    /// Current best block hash
    pub best_hash: Option<String>,
}

impl ChainState {
    pub fn new() -> Self {
        Self {
            blocks: HashMap::new(),
            best_hash: None,
        }
    }

    pub fn add_block(&mut self, block: BlockNode) {
        self.blocks.insert(block.hash.clone(), block);
    }

    pub fn get_all_blocks(&self) -> Vec<BlockNode> {
        self.blocks.values().cloned().collect()
    }
}

/// Poll Bitcoin Core every second to detect new blocks and chain state changes
pub async fn poll_chain(state: Arc<AppState>) {
    let mut last_best = String::new();
    let mut last_tip_count: usize = 0;
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));

    loop {
        interval.tick().await;

        let best_hash = match state.rpc.get_best_block_hash().await {
            Ok(h) => h,
            Err(e) => {
                tracing::warn!("Failed to get best block hash: {e}");
                continue;
            }
        };

        let tip_count = state
            .rpc
            .get_chain_tips()
            .await
            .map(|t| t.len())
            .unwrap_or(0);

        if best_hash == last_best && tip_count == last_tip_count {
            continue;
        }

        tracing::info!("Chain update: best={} tips={}", &best_hash[..16], tip_count);
        last_best = best_hash.clone();
        last_tip_count = tip_count;

        // Sync the full chain from tips
        if let Err(e) = sync_chain(&state).await {
            tracing::error!("Failed to sync chain: {e}");
            continue;
        }

        // Broadcast new block event
        let _ = state.tx.send(BlockEvent::NewBlock {
            hash: best_hash.clone(),
        });
    }
}

/// Walk back from all chain tips and rebuild the block tree
async fn sync_chain(state: &Arc<AppState>) -> Result<(), String> {
    let tips = state.rpc.get_chain_tips().await?;
    let best_hash = state.rpc.get_best_block_hash().await?;

    // Rebuild the entire block map from all tips
    let mut new_blocks = HashMap::new();

    for tip in &tips {
        let mut hash = tip.hash.clone();

        loop {
            // Skip if already fetched in this sync pass
            if new_blocks.contains_key(&hash) {
                break;
            }

            let block = match state.rpc.get_block(&hash).await {
                Ok(b) => b,
                Err(e) => {
                    tracing::warn!("Failed to get block {hash}: {e}");
                    break;
                }
            };

            let node = BlockNode::from(&block);
            let prev = node.prev_hash.clone();
            new_blocks.insert(hash, node);

            match prev {
                Some(p) => hash = p,
                None => break, // genesis
            }
        }
    }

    // Mark active chain by walking from best tip
    for block in new_blocks.values_mut() {
        block.is_active = false;
    }
    let mut h = best_hash.clone();
    loop {
        if let Some(block) = new_blocks.get_mut(&h) {
            block.is_active = true;
            match block.prev_hash.clone() {
                Some(prev) => h = prev,
                None => break,
            }
        } else {
            break;
        }
    }

    // Swap in the new state
    let mut chain = state.chain.lock().await;
    chain.blocks = new_blocks;
    chain.best_hash = Some(best_hash);

    Ok(())
}
