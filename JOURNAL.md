# Orphan - Development Journal

## Day 1: Project Setup & Foundation

### What I built
- **Rust backend** with Axum web framework (edition 2024)
  - Bitcoin Core RPC client (`rpc.rs`) - full JSON-RPC integration
  - Chain state tracker (`chain.rs`) - block tree with fork detection
  - WebSocket handler (`ws.rs`) - real-time block event broadcasting
  - REST API (`api.rs`) - chain tips, block tree, block details, mining endpoints
  - Fork creation via `invalidateblock`/`reconsiderblock` pattern

- **Next.js frontend** with Tailwind CSS
  - SVG-based chain visualization (custom, not React Flow)
  - Interactive block nodes with click-to-select and click-to-mine
  - Block details sidebar panel
  - WebSocket connection for real-time updates
  - Educational "How it works" overlay
  - Dark theme with Bitcoin orange accent

- **Docker Compose** for one-command local setup
  - Bitcoin Core (regtest) container
  - Rust backend container

### Architecture decisions
- **SVG over React Flow**: More control, lighter weight, unique approach vs competitors
- **Axum over Actix**: Modern, tower-based, great WebSocket support
- **Polling pattern**: 1-second poll of Bitcoin Core rather than ZMQ (simpler, good enough for regtest)
- **Fork creation**: Using `invalidateblock`/`reconsiderblock` RPC calls to create real forks in Bitcoin Core's block tree

### Key API endpoints
```
GET  /api/health          - Health check
GET  /api/chain/info      - Blockchain info
GET  /api/chain/tips      - All chain tips
GET  /api/chain/blocks    - Full block tree
GET  /api/block/:hash     - Block details
POST /api/mine            - Mine on best chain
POST /api/mine/:hash      - Mine extending specific block (creates forks!)
WS   /ws                  - Real-time block events
```
