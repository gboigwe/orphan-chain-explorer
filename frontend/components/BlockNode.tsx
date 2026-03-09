"use client";

import { BlockNode as BlockNodeType } from "@/lib/api";

interface Props {
  block: BlockNodeType;
  x: number;
  y: number;
  selected: boolean;
  isTip: boolean;
  onClick: () => void;
  onMine: () => void;
}

const BLOCK_W = 120;
const BLOCK_H = 56;

export default function BlockNodeComponent({
  block,
  x,
  y,
  selected,
  isTip,
  onClick,
  onMine,
}: Props) {
  const isGenesis = block.height === 0;
  const fill = block.is_active ? "#16a34a" : "#dc2626";
  const stroke = selected ? "#f7931a" : block.is_active ? "#22c55e" : "#ef4444";
  const strokeWidth = selected ? 2.5 : 1.5;

  return (
    <g
      transform={`translate(${x}, ${y})`}
      onClick={onClick}
      className="cursor-pointer"
    >
      {/* Block rectangle */}
      <rect
        width={BLOCK_W}
        height={BLOCK_H}
        rx={8}
        ry={8}
        fill={fill}
        fillOpacity={0.15}
        stroke={stroke}
        strokeWidth={strokeWidth}
      />

      {/* Height label */}
      <text
        x={BLOCK_W / 2}
        y={20}
        textAnchor="middle"
        fill="#e5e5e5"
        fontSize={14}
        fontWeight={600}
        fontFamily="var(--font-geist-mono)"
      >
        {isGenesis ? "Genesis" : `#${block.height}`}
      </text>

      {/* Truncated hash */}
      <text
        x={BLOCK_W / 2}
        y={38}
        textAnchor="middle"
        fill="#888"
        fontSize={10}
        fontFamily="var(--font-geist-mono)"
      >
        {block.hash.slice(0, 8)}...
      </text>

      {/* Tx count badge */}
      <text
        x={BLOCK_W / 2}
        y={51}
        textAnchor="middle"
        fill="#666"
        fontSize={9}
      >
        {block.n_tx} tx
      </text>

      {/* Mine button for tips */}
      {isTip && (
        <g
          transform={`translate(${BLOCK_W + 8}, ${BLOCK_H / 2 - 12})`}
          onClick={(e) => {
            e.stopPropagation();
            onMine();
          }}
          className="cursor-pointer"
        >
          <rect
            width={24}
            height={24}
            rx={4}
            fill="#f7931a"
            fillOpacity={0.2}
            stroke="#f7931a"
            strokeWidth={1}
          />
          <text
            x={12}
            y={16}
            textAnchor="middle"
            fill="#f7931a"
            fontSize={14}
            fontWeight={700}
          >
            +
          </text>
        </g>
      )}
    </g>
  );
}

export { BLOCK_W, BLOCK_H };
