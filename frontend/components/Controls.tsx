"use client";

import { useState } from "react";
import { mineBlock } from "@/lib/api";

interface Props {
  connected: boolean;
  onRefresh: () => void;
}

export default function Controls({ connected, onRefresh }: Props) {
  const [mining, setMining] = useState(false);

  const handleMine = async () => {
    if (mining) return;
    setMining(true);
    try {
      await mineBlock();
      onRefresh();
    } catch (err) {
      console.error("Mining failed:", err);
    } finally {
      setMining(false);
    }
  };

  return (
    <div className="flex items-center gap-3">
      <button
        onClick={handleMine}
        disabled={mining}
        className="px-4 py-2 bg-[var(--accent)] text-black font-semibold rounded-lg
                   hover:bg-[var(--accent-dim)] disabled:opacity-50 disabled:cursor-not-allowed
                   transition-colors text-sm"
      >
        {mining ? "Mining..." : "Mine Block"}
      </button>

      <div className="flex items-center gap-1.5 text-xs text-zinc-500">
        <span
          className={`w-2 h-2 rounded-full ${
            connected ? "bg-green-500" : "bg-red-500"
          }`}
        />
        {connected ? "Connected" : "Disconnected"}
      </div>
    </div>
  );
}
