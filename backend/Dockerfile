FROM rust:1.85-slim AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/orphan-backend /usr/local/bin/orphan-backend

ENV BITCOIN_RPC_URL=http://bitcoind:18443
ENV BITCOIN_RPC_USER=bitcoin
ENV BITCOIN_RPC_PASS=bitcoin
ENV LISTEN_ADDR=0.0.0.0:3001

EXPOSE 3001

CMD ["orphan-backend"]
