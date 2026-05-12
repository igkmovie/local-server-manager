import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { ServerView } from "../types/server";
import { StatusBadge } from "./StatusBadge";

type Props = {
  server?: ServerView;
  onChanged: () => void;
};

export function ServerDetail({ server, onChanged }: Props) {
  const [busy, setBusy] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  if (!server) {
    return (
      <div className="text-sm text-neutral-500 italic">Select a server.</div>
    );
  }

  const cmd = `${server.command} ${server.args.join(" ")}`.trim();
  const isManaged =
    server.pid !== undefined ||
    server.status === "running" ||
    server.status === "starting" ||
    server.status === "unhealthy" ||
    server.status === "stopping";

  async function run(name: string, fn: () => Promise<unknown>) {
    setBusy(name);
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
    <div className="space-y-2 text-sm">
      <div className="flex items-center gap-2">
        <h2 className="text-base font-semibold">{server.name}</h2>
        <StatusBadge status={server.status} />
      </div>
      <Row label="ID" value={server.id} />
      <Row label="Command" value={cmd} mono />
      <Row label="CWD" value={server.cwd} mono />
      {server.port !== undefined && <Row label="Port" value={String(server.port)} />}
      {server.healthUrl && <Row label="Health URL" value={server.healthUrl} mono />}
      {server.openUrl && <Row label="Open URL" value={server.openUrl} mono />}
      {server.pid !== undefined && <Row label="PID" value={String(server.pid)} />}
      <div className="flex flex-wrap gap-2 pt-2">
        <Action
          color="green"
          disabled={!!busy || isManaged}
          loading={busy === "start"}
          onClick={() =>
            run("start", () =>
              invoke<number>("start_server", { serverId: server.id }),
            )
          }
        >
          Start
        </Action>
        <Action
          color="red"
          disabled={!!busy || !isManaged}
          loading={busy === "stop"}
          onClick={() =>
            run("stop", () =>
              invoke<void>("stop_server", { serverId: server.id }),
            )
          }
        >
          Stop
        </Action>
        {server.status === "external" && (
          <Action
            color="red"
            disabled={!!busy}
            loading={busy === "forceStop"}
            onClick={() => {
              const ok = window.confirm(
                `This will kill the external process listening on port ${server.port}.\n\nWarning: if the process is writing data (DB, file I/O), force-killing may cause corruption.\n\nContinue?`,
              );
              if (!ok) return;
              run("forceStop", () =>
                invoke<number>("force_stop_external", {
                  serverId: server.id,
                }),
              );
            }}
          >
            Force Stop (external)
          </Action>
        )}
        <Action
          color="blue"
          disabled={!!busy}
          loading={busy === "restart"}
          onClick={() =>
            run("restart", () =>
              invoke<number>("restart_server", { serverId: server.id }),
            )
          }
        >
          Restart
        </Action>
        <Action
          color="neutral"
          disabled={!!busy || !server.openUrl}
          loading={busy === "openUrl"}
          onClick={() =>
            run("openUrl", () =>
              invoke<void>("open_server_url", { serverId: server.id }),
            )
          }
        >
          Open URL
        </Action>
        <Action
          color="neutral"
          disabled={!!busy}
          loading={busy === "revealLog"}
          onClick={() =>
            run("revealLog", () =>
              invoke<void>("reveal_log_file", { serverId: server.id }),
            )
          }
        >
          Reveal Log
        </Action>
      </div>
      {error && (
        <div className="rounded border border-red-800 bg-red-950/50 px-2 py-1 font-mono text-xs text-red-200">
          {error}
        </div>
      )}
    </div>
  );
}

function Row({
  label,
  value,
  mono,
}: {
  label: string;
  value: string;
  mono?: boolean;
}) {
  return (
    <div className="grid grid-cols-[110px_1fr] gap-2">
      <div className="text-neutral-500">{label}</div>
      <div
        className={`break-all ${mono ? "font-mono text-xs text-neutral-300" : "text-neutral-200"}`}
      >
        {value}
      </div>
    </div>
  );
}

type ActionColor = "green" | "red" | "blue" | "neutral";

const COLOR_CLASSES: Record<ActionColor, string> = {
  green: "border-green-700 bg-green-900/40 text-green-200 hover:bg-green-900/70",
  red: "border-red-700 bg-red-900/40 text-red-200 hover:bg-red-900/70",
  blue: "border-blue-700 bg-blue-900/40 text-blue-200 hover:bg-blue-900/70",
  neutral: "border-neutral-700 text-neutral-400",
};

function Action({
  children,
  color,
  disabled,
  loading,
  onClick,
}: {
  children: React.ReactNode;
  color: ActionColor;
  disabled?: boolean;
  loading?: boolean;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      disabled={disabled}
      className={`rounded border px-3 py-1 text-xs disabled:opacity-40 ${COLOR_CLASSES[color]}`}
    >
      {loading ? "…" : children}
    </button>
  );
}
