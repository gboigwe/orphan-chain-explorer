"use client";

import { useCallback, useMemo, useRef, useState } from "react";
import { BlockNode as BlockNodeType, mineOnBlock } from "@/lib/api";
import BlockNodeComponent, { BLOCK_W, BLOCK_H } from "./BlockNode";

interface Props {
  blocks: BlockNodeType[];
  selectedHash: string | null;
  onSelect: (hash: string) => void;
  onRefresh: () => void;
}

const H_GAP = 40; // horizontal gap between blocks
const V_GAP = 24; // vertical gap between branches
const PADDING = 40;

interface LayoutNode {
  block: BlockNodeType;
  x: number;
  y: number;
  isTip: boolean;
}

export default function ChainGraph({
  blocks,
  selectedHash,
  onSelect,
  onRefresh,
}: Props) {
  const svgRef = useRef<SVGSVGElement>(null);
  const [mining, setMining] = useState(false);

  // Build a map from hash to block and from hash to children
  const { byHash, children } = useMemo(() => {
    const byHash = new Map<string, BlockNodeType>();
    const children = new Map<string, string[]>();

    for (const b of blocks) {
      byHash.set(b.hash, b);
      if (b.prev_hash) {
        const c = children.get(b.prev_hash) || [];
        c.push(b.hash);
        children.set(b.prev_hash, c);
      }
    }
    return { byHash, children };
  }, [blocks]);

  // Compute layout positions
  const { nodes, edges, svgWidth, svgHeight } = useMemo(() => {
    if (blocks.length === 0) {
      return { nodes: [], edges: [], svgWidth: 400, svgHeight: 200 };
    }

    // Find genesis (block with no prev_hash or prev_hash not in our set)
    const genesis = blocks.find(
      (b) => !b.prev_hash || !byHash.has(b.prev_hash)
    );
    if (!genesis) {
      return { nodes: [], edges: [], svgWidth: 400, svgHeight: 200 };
    }

    // Find all tips (blocks with no children)
    const tipSet = new Set<string>();
    for (const b of blocks) {
      if (!children.has(b.hash) || children.get(b.hash)!.length === 0) {
        tipSet.add(b.hash);
      }
    }

    // BFS to assign positions
    // Each height level gets an x position
    // Multiple blocks at the same height get different y (branch) positions
    const nodes: LayoutNode[] = [];
    const edges: { x1: number; y1: number; x2: number; y2: number; active: boolean }[] = [];

    // Track branch index per height level
    const heightBranches = new Map<number, number>();

    // DFS from genesis
    const visited = new Set<string>();

    function dfs(hash: string, branchOffset: number) {
      if (visited.has(hash)) return;
      visited.add(hash);

      const block = byHash.get(hash);
      if (!block) return;

      // Get branch index at this height
      const currentBranch = heightBranches.get(block.height) || 0;
      heightBranches.set(block.height, currentBranch + 1);

      const x = PADDING + block.height * (BLOCK_W + H_GAP);
      const y = PADDING + (branchOffset + currentBranch) * (BLOCK_H + V_GAP);

      const isTip = tipSet.has(hash);
      nodes.push({ block, x, y, isTip });

      // Draw edge from parent
      if (block.prev_hash) {
        const parentNode = nodes.find((n) => n.block.hash === block.prev_hash);
        if (parentNode) {
          edges.push({
            x1: parentNode.x + BLOCK_W,
            y1: parentNode.y + BLOCK_H / 2,
            x2: x,
            y2: y + BLOCK_H / 2,
            active: block.is_active && parentNode.block.is_active,
          });
        }
      }

      // Process children - active chain first
      const childHashes = children.get(hash) || [];
      const sorted = childHashes.sort((a, b) => {
        const ba = byHash.get(a);
        const bb = byHash.get(b);
        if (ba?.is_active && !bb?.is_active) return -1;
        if (!ba?.is_active && bb?.is_active) return 1;
        return 0;
      });

      for (let i = 0; i < sorted.length; i++) {
        dfs(sorted[i], i > 0 ? i : 0);
      }
    }

    dfs(genesis.hash, 0);

    // Calculate SVG dimensions
    const maxX = Math.max(...nodes.map((n) => n.x)) + BLOCK_W + 60;
    const maxY = Math.max(...nodes.map((n) => n.y)) + BLOCK_H + PADDING;

    return {
      nodes,
      edges,
      svgWidth: Math.max(400, maxX),
      svgHeight: Math.max(200, maxY),
    };
  }, [blocks, byHash, children]);

  const handleMine = useCallback(
    async (hash: string) => {
      if (mining) return;
      setMining(true);
      try {
        await mineOnBlock(hash);
        onRefresh();
      } catch (err) {
        console.error("Mining failed:", err);
      } finally {
        setMining(false);
      }
    },
    [mining, onRefresh]
  );

  if (blocks.length === 0) {
    return (
      <div className="flex items-center justify-center h-full text-zinc-500">
        No blocks loaded. Is the backend running?
      </div>
    );
  }

  return (
    <div className="overflow-auto w-full h-full">
      <svg
        ref={svgRef}
        width={svgWidth}
        height={svgHeight}
        className="min-w-full"
      >
        {/* Edges */}
        {edges.map((e, i) => (
          <line
            key={i}
            x1={e.x1}
            y1={e.y1}
            x2={e.x2}
            y2={e.y2}
            stroke={e.active ? "#22c55e" : "#ef4444"}
            strokeWidth={2}
            strokeOpacity={0.5}
          />
        ))}

        {/* Block nodes */}
        {nodes.map((n) => (
          <BlockNodeComponent
            key={n.block.hash}
            block={n.block}
            x={n.x}
            y={n.y}
            selected={n.block.hash === selectedHash}
            isTip={n.isTip}
            onClick={() => onSelect(n.block.hash)}
            onMine={() => handleMine(n.block.hash)}
          />
        ))}
      </svg>
    </div>
  );
}
