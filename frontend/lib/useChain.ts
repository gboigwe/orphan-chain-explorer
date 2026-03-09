"use client";

import { useCallback, useEffect, useRef, useState } from "react";
import { BlockNode, fetchChainBlocks, getWebSocketUrl } from "./api";

export function useChain() {
  const [blocks, setBlocks] = useState<BlockNode[]>([]);
  const [connected, setConnected] = useState(false);
  const [loading, setLoading] = useState(true);
  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimer = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);

  const loadBlocks = useCallback(async () => {
    try {
      const data = await fetchChainBlocks();
      setBlocks(data);
    } catch {
      // Backend not available yet
    } finally {
      setLoading(false);
    }
  }, []);

  const connectWs = useCallback(() => {
    if (wsRef.current?.readyState === WebSocket.OPEN) return;

    const ws = new WebSocket(getWebSocketUrl());
    wsRef.current = ws;

    ws.onopen = () => {
      setConnected(true);
    };

    ws.onmessage = (event) => {
      const data = JSON.parse(event.data);

      if (data.type === "initial_state") {
        setBlocks(data.blocks);
        setLoading(false);
      } else if (
        data.type === "new_block" ||
        data.type === "reorg" ||
        data.type === "chain_update"
      ) {
        // Refresh the full block list on any chain event
        loadBlocks();
      }
    };

    ws.onclose = () => {
      setConnected(false);
      // Reconnect after 2 seconds
      reconnectTimer.current = setTimeout(connectWs, 2000);
    };

    ws.onerror = () => {
      ws.close();
    };
  }, [loadBlocks]);

  useEffect(() => {
    loadBlocks();
    connectWs();

    return () => {
      clearTimeout(reconnectTimer.current);
      wsRef.current?.close();
    };
  }, [loadBlocks, connectWs]);

  return { blocks, connected, loading, refresh: loadBlocks };
}
