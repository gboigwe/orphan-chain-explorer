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

        if best_hash == last_best {
            continue;
        }

        tracing::info!("New best block: {best_hash}");
        last_best = best_hash.clone();

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

/// Walk back from all chain tips and populate the block tree
async fn sync_chain(state: &Arc<AppState>) -> Result<(), String> {
    let tips = state.rpc.get_chain_tips().await?;
    let mut chain = state.chain.lock().await;

    let best_hash = state.rpc.get_best_block_hash().await?;
    chain.best_hash = Some(best_hash);

    for tip in &tips {
        let mut hash = tip.hash.clone();

        // Walk backwards from this tip until we reach a block we already know
        loop {
            if chain.blocks.contains_key(&hash) {
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
            chain.add_block(node);

            match prev {
                Some(p) => hash = p,
                None => break, // genesis
            }
        }
    }

    // Update is_active status for all blocks based on current best chain
    let best = chain.best_hash.clone();
    if let Some(best) = best {
        // First mark all as inactive
        for block in chain.blocks.values_mut() {
            block.is_active = false;
        }
        // Walk from best tip back to genesis marking active
        let mut h = best;
        loop {
            if let Some(block) = chain.blocks.get_mut(&h) {
                block.is_active = true;
                match block.prev_hash.clone() {
                    Some(prev) => h = prev,
                    None => break,
                }
            } else {
                break;
            }
        }
    }

    Ok(())
}
