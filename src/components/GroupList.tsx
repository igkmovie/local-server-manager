import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { GroupView } from "../types/server";

type Props = {
  groups: GroupView[];
  onChanged: () => void;
};

export function GroupList({ groups, onChanged }: Props) {
  const [busy, setBusy] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  if (groups.length === 0) {
    return <div className="text-sm text-neutral-500 italic">(no groups)</div>;
  }

  async function run(key: string, fn: () => Promise<unknown>) {
    setBusy(key);
    setError(null);
    try {
      await fn();
      onChanged();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(null);
    }
  }

  return (
    <div className="space-y-2">
      {error && (
        <div className="rounded border border-red-800 bg-red-950/50 px-2 py-1 font-mono text-[10px] text-red-200">
          {error}
        </div>
      )}
      <ul className="space-y-1">
        {groups.map((g) => {
          const startKey = `start:${g.id}`;
          const stopKey = `stop:${g.id}`;
          return (
            <li
              key={g.id}
              className="rounded border border-neutral-800 px-2 py-1.5 text-sm"
            >
              <div className="truncate text-neutral-200">{g.name}</div>
              <div className="mb-1 text-[10px] text-neutral-500">
                {g.serverIds.length} servers
              </div>
              <div className="flex gap-1">
                <button
                  type="button"
                  disabled={!!busy}
                  onClick={() =>
                    run(startKey, () =>
                      invoke("start_group", { groupId: g.id }),
                    )
                  }
                  className="flex-1 rounded border border-green-700 bg-green-900/30 px-1 py-0.5 text-[10px] text-green-200 hover:bg-green-900/60 disabled:opacity-40"
                >
                  {busy === startKey ? "…" : "Start all"}
                </button>
                <button
                  type="button"
                  disabled={!!busy}
                  onClick={() =>
                    run(stopKey, () =>
                      invoke("stop_group", { groupId: g.id }),
                    )
                  }
                  className="flex-1 rounded border border-red-700 bg-red-900/30 px-1 py-0.5 text-[10px] text-red-200 hover:bg-red-900/60 disabled:opacity-40"
                >
                  {busy === stopKey ? "…" : "Stop all"}
                </button>
              </div>
            </li>
          );
        })}
      </ul>
    </div>
  );
}
