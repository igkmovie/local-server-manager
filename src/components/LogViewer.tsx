import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

type Props = {
  serverId?: string;
};

export function LogViewer({ serverId }: Props) {
  const [text, setText] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [autoFollow, setAutoFollow] = useState(true);
  const preRef = useRef<HTMLPreElement>(null);

  async function load() {
    if (!serverId) {
      setText("");
      return;
    }
    try {
      const t = await invoke<string>("get_server_log", {
        serverId,
        lines: 500,
      });
      setText(t);
      setError(null);
    } catch (e) {
      setError(String(e));
    }
  }

  useEffect(() => {
    load();
    const id = setInterval(load, 1500);
    return () => clearInterval(id);
  }, [serverId]);

  useEffect(() => {
    if (autoFollow && preRef.current) {
      preRef.current.scrollTop = preRef.current.scrollHeight;
    }
  }, [text, autoFollow]);

  if (!serverId) {
    return (
      <div className="text-sm text-neutral-500 italic">
        Select a server to view logs.
      </div>
    );
  }

  return (
    <div className="flex h-full flex-col">
      <div className="mb-2 flex items-center gap-2 text-xs text-neutral-400">
        <label className="flex items-center gap-1">
          <input
            type="checkbox"
            checked={autoFollow}
            onChange={(e) => setAutoFollow(e.target.checked)}
          />
          Follow tail
        </label>
        <button
          type="button"
          onClick={load}
          className="rounded border border-neutral-700 px-2 py-0.5 text-[10px] hover:bg-neutral-800"
        >
          Reload
        </button>
      </div>
      {error && (
        <div className="mb-2 rounded border border-red-800 bg-red-950/50 px-2 py-1 font-mono text-xs text-red-200">
          {error}
        </div>
      )}
      <pre
        ref={preRef}
        className="flex-1 overflow-auto whitespace-pre-wrap break-all rounded border border-neutral-800 bg-black/40 p-2 font-mono text-[11px] leading-relaxed text-neutral-300"
      >
        {text || "(empty)"}
      </pre>
    </div>
  );
}
