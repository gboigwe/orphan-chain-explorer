"use client";

import { useEffect, useState } from "react";
import { BlockDetail, fetchBlockDetail } from "@/lib/api";

interface Props {
  hash: string | null;
  onClose: () => void;
}

export default function BlockPanel({ hash, onClose }: Props) {
  const [block, setBlock] = useState<BlockDetail | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (!hash) {
      setBlock(null);
      return;
    }

    setLoading(true);
    fetchBlockDetail(hash)
      .then(setBlock)
      .catch(() => setBlock(null))
      .finally(() => setLoading(false));
  }, [hash]);

  if (!hash) return null;

  return (
    <div className="w-80 border-l border-zinc-800 bg-[var(--panel-bg)] p-4 overflow-y-auto flex-shrink-0">
      <div className="flex justify-between items-center mb-4">
        <h2 className="text-lg font-semibold text-[var(--accent)]">
          Block Details
        </h2>
        <button
          onClick={onClose}
          className="text-zinc-500 hover:text-zinc-300 text-xl leading-none"
        >
          x
        </button>
      </div>

      {loading && <p className="text-zinc-500">Loading...</p>}

      {block && (
        <dl className="space-y-3 font-mono text-sm">
          <Field label="Height" value={block.height.toString()} />
          <Field label="Hash" value={block.hash} mono />
          <Field label="Previous" value={block.previousblockhash || "None"} mono />
          <Field
            label="Time"
            value={new Date(block.time * 1000).toLocaleString()}
          />
          <Field label="Transactions" value={block.nTx.toString()} />
          <Field label="Size" value={`${block.size.toLocaleString()} bytes`} />
          <Field
            label="Weight"
            value={`${block.weight.toLocaleString()} WU`}
          />
          <Field label="Version" value={`0x${block.version.toString(16)}`} />
          <Field label="Nonce" value={block.nonce.toString()} />
          <Field label="Bits" value={block.bits} />
          <Field label="Difficulty" value={block.difficulty.toString()} />
          <Field label="Merkle Root" value={block.merkleroot} mono />
          <Field
            label="Confirmations"
            value={block.confirmations.toString()}
            highlight={block.confirmations < 0}
          />
        </dl>
      )}
    </div>
  );
}

function Field({
  label,
  value,
  mono,
  highlight,
}: {
  label: string;
  value: string;
  mono?: boolean;
  highlight?: boolean;
}) {
  return (
    <div>
      <dt className="text-zinc-500 text-xs uppercase tracking-wider">
        {label}
      </dt>
      <dd
        className={`mt-0.5 break-all ${mono ? "text-xs" : ""} ${
          highlight ? "text-red-400" : "text-zinc-200"
        }`}
      >
        {value}
      </dd>
    </div>
  );
}
