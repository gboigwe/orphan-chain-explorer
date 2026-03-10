# Orphan - Development Journal

## March 3 - Getting started

Picked the visual chain/reorg project from 0xB10C's ideas. Set up the repo, wrote the README with the planned architecture. Going with Rust (Axum) for the backend and Next.js for the frontend.

Main question right now: how to actually create forks in regtest? Need to dig into Bitcoin Core's RPC docs.

## March 4 - Backend scaffold

Got the basic Axum server running with health check endpoint. Set up Cargo project with the dependencies I think I'll need: axum, tokio, serde, reqwest for RPC calls.

Still figuring out the best way to structure the RPC client. Bitcoin Core uses JSON-RPC 2.0 with basic auth.

## March 5 - RPC client and chain tracking

Built the RPC client module. Can now call `getblockchaininfo`, `getblock`, `getchaintips`, `generatetoaddress`. Also started on the chain state tracker that builds a block tree in memory by walking back from chain tips.

Learned that `getchaintips` returns all tips including stale ones — that's exactly what I need for showing forks.

## March 6 - API and WebSocket

Added REST endpoints for getting chain tips, block tree, block details. Also the mining endpoints.

For creating forks, my plan is: `invalidateblock` on the current chain to make the target block the tip, mine there, then `reconsiderblock`. Need to test if this actually works.

Set up WebSocket handler with tokio broadcast channel. The idea is the backend polls Bitcoin Core every second and pushes events to connected clients.

## March 7 - Docker setup

Created docker-compose.yml with Bitcoin Core (regtest) and the Rust backend. Wrote a multi-stage Dockerfile for the backend build.

Haven't tested the full Docker flow yet — need to make sure the containers can talk to each other.

## March 8-9 - Frontend

Initialized Next.js project. Built the SVG-based chain visualization — each block is a node, connected by lines to its parent. Green for active chain, red for stale blocks.

The layout algorithm uses DFS from genesis, assigning x position by height and y position by branch. It's basic but works for the block counts we'll have on regtest.

Also added a block details panel, mining controls, and a small explainer overlay.

## What's next

- Actually test the full flow end to end with Docker
- See if the fork creation via invalidateblock/reconsiderblock works
- Fix any issues that come up when connecting frontend to backend
- Make the WebSocket updates reliable
