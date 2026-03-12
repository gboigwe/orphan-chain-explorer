use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone)]
pub struct BitcoinRpc {
    url: String,
    auth: String, // base64-encoded user:pass
    client: reqwest::Client,
    id_counter: std::sync::Arc<AtomicU64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct RpcRequest {
    jsonrpc: String,
    id: u64,
    method: String,
    params: Value,
}

#[derive(Debug, Deserialize)]
struct RpcResponse {
    result: Option<Value>,
    error: Option<Value>,
    #[allow(dead_code)]
    id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockInfo {
    pub hash: String,
    pub height: u64,
    pub version: i64,
    pub time: u64,
    pub nonce: u64,
    pub bits: String,
    pub difficulty: f64,
    pub previousblockhash: Option<String>,
    pub nextblockhash: Option<String>,
    pub merkleroot: String,
    #[serde(rename = "nTx")]
    pub n_tx: u64,
    pub size: u64,
    pub weight: u64,
    pub confirmations: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainTip {
    pub height: u64,
    pub hash: String,
    pub branchlen: u64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainInfo {
    pub chain: String,
    pub blocks: u64,
    pub bestblockhash: String,
}

impl BitcoinRpc {
    pub fn new(url: &str, user: &str, pass: &str) -> Self {
        use base64::prelude::*;
        let auth_str = format!("{user}:{pass}");
        let auth = BASE64_STANDARD.encode(auth_str.as_bytes());
        Self {
            url: url.to_string(),
            auth,
            client: reqwest::Client::new(),
            id_counter: std::sync::Arc::new(AtomicU64::new(0)),
        }
    }

    async fn call(&self, method: &str, params: Value) -> Result<Value, String> {
        let id = self.id_counter.fetch_add(1, Ordering::Relaxed);
        let req = RpcRequest {
            jsonrpc: "2.0".to_string(),
            id,
            method: method.to_string(),
            params,
        };

        let resp = self
            .client
            .post(&self.url)
            .header("Authorization", format!("Basic {}", self.auth))
            .header("Content-Type", "application/json")
            .json(&req)
            .send()
            .await
            .map_err(|e| format!("RPC request failed: {e}"))?;

        let rpc_resp: RpcResponse = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse RPC response: {e}"))?;

        if let Some(err) = rpc_resp.error {
            return Err(format!("RPC error: {err}"));
        }

        Ok(rpc_resp.result.unwrap_or(Value::Null))
    }

    pub async fn get_blockchain_info(&self) -> Result<BlockchainInfo, String> {
        let val = self.call("getblockchaininfo", json!([])).await?;
        serde_json::from_value(val).map_err(|e| format!("Parse error: {e}"))
    }

    pub async fn get_best_block_hash(&self) -> Result<String, String> {
        let val = self.call("getbestblockhash", json!([])).await?;
        val.as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| "Expected string".into())
    }

    pub async fn get_block_hash(&self, height: u64) -> Result<String, String> {
        let val = self.call("getblockhash", json!([height])).await?;
        val.as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| "Expected string".into())
    }

    pub async fn get_block(&self, hash: &str) -> Result<BlockInfo, String> {
        // verbosity=1 returns JSON object
        let val = self.call("getblock", json!([hash, 1])).await?;
        serde_json::from_value(val).map_err(|e| format!("Parse error: {e}"))
    }

    pub async fn get_chain_tips(&self) -> Result<Vec<ChainTip>, String> {
        let val = self.call("getchaintips", json!([])).await?;
        serde_json::from_value(val).map_err(|e| format!("Parse error: {e}"))
    }

    /// Mine `count` blocks. If `parent_hash` is provided, we invalidate
    /// the current tip and reconsider blocks to mine on a specific fork.
    pub async fn generate_to_address(
        &self,
        count: u64,
        address: &str,
    ) -> Result<Vec<String>, String> {
        let val = self
            .call("generatetoaddress", json!([count, address]))
            .await?;
        serde_json::from_value(val).map_err(|e| format!("Parse error: {e}"))
    }

    pub async fn get_new_address(&self) -> Result<String, String> {
        let val = self.call("getnewaddress", json!([])).await?;
        val.as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| "Expected string".into())
    }

    /// Ensure a wallet exists, create one if not
    pub async fn ensure_wallet(&self) -> Result<(), String> {
        let wallets = self.call("listwallets", json!([])).await?;
        if let Some(arr) = wallets.as_array() {
            if !arr.is_empty() {
                return Ok(());
            }
        }
        // Try to create a default wallet
        match self.call("createwallet", json!(["default"])).await {
            Ok(_) => Ok(()),
            Err(e) if e.contains("already exists") => Ok(()),
            Err(e) => Err(e),
        }
    }

    /// Invalidate a block (marks it and descendants as invalid)
    pub async fn invalidate_block(&self, hash: &str) -> Result<(), String> {
        self.call("invalidateblock", json!([hash])).await?;
        Ok(())
    }

    /// Reconsider a previously invalidated block
    pub async fn reconsider_block(&self, hash: &str) -> Result<(), String> {
        self.call("reconsiderblock", json!([hash])).await?;
        Ok(())
    }

    /// Submit a block hex
    pub async fn submit_block(&self, hex: &str) -> Result<Value, String> {
        self.call("submitblock", json!([hex])).await
    }

    /// Get block in raw hex (verbosity=0)
    pub async fn get_block_hex(&self, hash: &str) -> Result<String, String> {
        let val = self.call("getblock", json!([hash, 0])).await?;
        val.as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| "Expected string".into())
    }
}
