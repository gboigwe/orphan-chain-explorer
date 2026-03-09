"use client";

import { useState } from "react";

export default function Explainer() {
  const [open, setOpen] = useState(false);

  return (
    <div className="relative">
      <button
        onClick={() => setOpen(!open)}
        className="px-3 py-2 text-xs border border-zinc-700 rounded-lg
                   hover:border-zinc-500 transition-colors text-zinc-400"
      >
        {open ? "Hide Guide" : "How it works"}
      </button>

      {open && (
        <div className="absolute top-10 right-0 w-80 bg-[var(--panel-bg)] border border-zinc-700 rounded-lg p-4 shadow-xl z-50">
          <h3 className="font-semibold text-[var(--accent)] mb-2">
            Chain Visualization Guide
          </h3>
          <ul className="space-y-2 text-xs text-zinc-400">
            <li className="flex items-start gap-2">
              <span className="w-3 h-3 rounded-sm bg-green-500/30 border border-green-500 mt-0.5 flex-shrink-0" />
              <span>
                <strong className="text-zinc-200">Green blocks</strong> are on
                the active (best) chain - the one with the most proof of work.
              </span>
            </li>
            <li className="flex items-start gap-2">
              <span className="w-3 h-3 rounded-sm bg-red-500/30 border border-red-500 mt-0.5 flex-shrink-0" />
              <span>
                <strong className="text-zinc-200">Red blocks</strong> are stale
                - they were on a competing chain that lost.
              </span>
            </li>
            <li className="flex items-start gap-2">
              <span className="w-3 h-3 rounded-sm bg-amber-500/30 border border-amber-500 mt-0.5 flex-shrink-0" />
              <span>
                <strong className="text-zinc-200">+ button</strong> on chain
                tips lets you mine a new block extending that specific chain.
              </span>
            </li>
            <li>
              <strong className="text-zinc-200">Reorg:</strong> When a competing
              chain gets more work than the active chain, Bitcoin Core switches
              to it. This is a &quot;reorganization&quot; - blocks that were
              active become stale, and stale blocks become active.
            </li>
            <li>
              <strong className="text-zinc-200">Try it:</strong> Mine a few
              blocks, then click + on an earlier block to create a fork. Keep
              mining on the fork until it overtakes the main chain!
            </li>
          </ul>
        </div>
      )}
    </div>
  );
}
