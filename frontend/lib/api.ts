export interface BlockNode {
  hash: string;
  height: number;
  prev_hash: string | null;
  time: number;
  n_tx: number;
  size: number;
  weight: number;
  is_active: boolean;
  confirmations: number;
}

export interface BlockDetail {
  hash: string;
  height: number;
  version: number;
  time: number;
  nonce: number;
  bits: string;
  difficulty: number;
  previousblockhash: string | null;
  nextblockhash: string | null;
  merkleroot: string;
  nTx: number;
  size: number;
  weight: number;
  confirmations: number;
}

export interface ChainTip {
  height: number;
  hash: string;
  branchlen: number;
  status: string;
}

const API_BASE =
  process.env.NEXT_PUBLIC_API_URL || "http://localhost:3001";

export async function fetchChainBlocks(): Promise<BlockNode[]> {
  const res = await fetch(`${API_BASE}/api/chain/blocks`);
  const data = await res.json();
  return data.blocks;
}

export async function fetchChainTips(): Promise<ChainTip[]> {
  const res = await fetch(`${API_BASE}/api/chain/tips`);
  const data = await res.json();
  return data.tips;
}

export async function fetchBlockDetail(hash: string): Promise<BlockDetail> {
  const res = await fetch(`${API_BASE}/api/block/${hash}`);
  return res.json();
}

export async function mineBlock(): Promise<{ mined: string[] }> {
  const res = await fetch(`${API_BASE}/api/mine`, { method: "POST" });
  return res.json();
}

export async function mineOnBlock(
  parentHash: string
): Promise<{ mined: string[]; fork_created: boolean }> {
  const res = await fetch(`${API_BASE}/api/mine/${parentHash}`, {
    method: "POST",
  });
  return res.json();
}

export function getWebSocketUrl(): string {
  const wsBase =
    process.env.NEXT_PUBLIC_WS_URL ||
    API_BASE.replace(/^http/, "ws");
  return `${wsBase}/ws`;
}
