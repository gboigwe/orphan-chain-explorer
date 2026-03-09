"use client";

import { useState } from "react";
import { useChain } from "@/lib/useChain";
import ChainGraph from "@/components/ChainGraph";
import BlockPanel from "@/components/BlockPanel";
import Controls from "@/components/Controls";
import Explainer from "@/components/Explainer";

export default function Home() {
  const { blocks, connected, loading, refresh } = useChain();
  const [selectedHash, setSelectedHash] = useState<string | null>(null);

  return (
    <div className="flex flex-col h-screen">
      {/* Header */}
      <header className="flex items-center justify-between px-6 py-3 border-b border-zinc-800">
        <div className="flex items-center gap-3">
          <h1 className="text-xl font-bold tracking-tight">
            <span className="text-[var(--accent)]">Orphan</span>
            <span className="text-zinc-400 font-normal ml-2 text-sm">
              Chain & Reorg Explorer
            </span>
          </h1>
        </div>

        <div className="flex items-center gap-4">
          <Explainer />
          <Controls connected={connected} onRefresh={refresh} />
        </div>
      </header>

      {/* Main content */}
      <div className="flex flex-1 overflow-hidden">
        {/* Chain visualization */}
        <main className="flex-1 overflow-auto bg-[var(--background)]">
          {loading ? (
            <div className="flex items-center justify-center h-full">
              <div className="text-zinc-500 flex flex-col items-center gap-3">
                <div className="w-8 h-8 border-2 border-zinc-600 border-t-[var(--accent)] rounded-full animate-spin" />
                <p>Connecting to backend...</p>
              </div>
            </div>
          ) : (
            <ChainGraph
              blocks={blocks}
              selectedHash={selectedHash}
              onSelect={setSelectedHash}
              onRefresh={refresh}
            />
          )}
        </main>

        {/* Block details sidebar */}
        <BlockPanel
          hash={selectedHash}
          onClose={() => setSelectedHash(null)}
        />
      </div>

      {/* Footer status bar */}
      <footer className="flex items-center justify-between px-6 py-1.5 border-t border-zinc-800 text-xs text-zinc-600">
        <span>{blocks.length} blocks loaded</span>
        <span>Regtest</span>
      </footer>
    </div>
  );
}
