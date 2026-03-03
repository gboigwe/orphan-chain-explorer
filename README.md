# Orphan - Visual Chain, Reorg & Fork Creator

An interactive visual tool for exploring Bitcoin chain forks and reorganizations on **Regtest** and **Signet**.

Built for the [BOSS Challenge 2026](https://bosschallenge.xyz/) based on [0xB10C's project idea #2](https://github.com/0xB10C/project-ideas/issues/2): "Visual chain, reorg, and fork creator for Regtest and Signet."

## Features

- **Visual Block Tree** - SVG-based chain visualization showing active chain and stale blocks
- **Interactive Mining** - Click to mine blocks on any chain tip, creating forks in real-time
- **Real-time Updates** - WebSocket connection pushes new blocks instantly to the UI
- **Fork Creation** - Mine on competing chains to trigger Bitcoin Core reorganizations
- **Block Inspector** - Click any block to see full details (hash, height, txs, merkle root, etc.)
- **Educational Mode** - Built-in explanations of how forks and reorgs work

## Quick Start

### Docker (recommended)

```bash
docker compose up
```

Then open [http://localhost:3000](http://localhost:3000).

### Manual Setup

**Prerequisites:** Rust, Node.js, Bitcoin Core (regtest)

1. Start Bitcoin Core in regtest mode:
```bash
bitcoind -regtest -server -rpcuser=bitcoin -rpcpassword=bitcoin -fallbackfee=0.0001
```

2. Start the backend:
```bash
cd backend
cargo run
```

3. Start the frontend:
```bash
cd frontend
npm install
npm run dev
```

4. Open [http://localhost:3000](http://localhost:3000)

## Architecture

```
┌──────────────┐   WebSocket/REST   ┌──────────────┐   JSON-RPC   ┌──────────────┐
│  Next.js App │◄──────────────────►│ Rust Backend │◄────────────►│ Bitcoin Core  │
│  (Port 3000) │                    │ (Port 3001)  │              │ (Regtest)     │
└──────────────┘                    └──────────────┘              └──────────────┘
```

## Tech Stack

- **Frontend:** Next.js, TypeScript, Tailwind CSS, Custom SVG visualization
- **Backend:** Rust, Axum, WebSockets
- **Bitcoin:** Bitcoin Core (Regtest/Signet)
- **Deployment:** Docker Compose

## How Forks Work

1. Mine a few blocks on the main chain
2. Click the **+** button on an earlier block to mine a competing chain
3. Keep mining on the fork - when it has more work than the main chain, Bitcoin Core will reorganize
4. Watch as green (active) and red (stale) blocks swap in real-time

## API

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/health` | GET | Health check |
| `/api/chain/info` | GET | Blockchain info |
| `/api/chain/tips` | GET | All chain tips |
| `/api/chain/blocks` | GET | Full block tree |
| `/api/block/:hash` | GET | Block details |
| `/api/mine` | POST | Mine on best chain |
| `/api/mine/:hash` | POST | Mine extending a specific block |
| `/ws` | WS | Real-time block events |

## License

MIT
