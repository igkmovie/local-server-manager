import { useEffect, useState } from "react";
import { check, Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

type Phase = "idle" | "checking" | "available" | "downloading" | "ready" | "error";

export function UpdateBanner() {
  const [phase, setPhase] = useState<Phase>("idle");
  const [update, setUpdate] = useState<Update | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [progress, setProgress] = useState<{ downloaded: number; total?: number }>({
    downloaded: 0,
  });

  async function doCheck() {
    setPhase("checking");
    setError(null);
    try {
      const u = await check();
      if (u) {
        setUpdate(u);
        setPhase("available");
      } else {
        setPhase("idle");
      }
    } catch (e) {
      setError(String(e));
      setPhase("error");
    }
  }

  useEffect(() => {
    doCheck();
  }, []);

  async function install() {
    if (!update) return;
    setPhase("downloading");
    setError(null);
    try {
      let total: number | undefined;
      let downloaded = 0;
      await update.downloadAndInstall((event) => {
        if (event.event === "Started") {
          total = event.data.contentLength;
          setProgress({ downloaded: 0, total });
        } else if (event.event === "Progress") {
          downloaded += event.data.chunkLength;
          setProgress({ downloaded, total });
        } else if (event.event === "Finished") {
          setPhase("ready");
        }
      });
      await relaunch();
    } catch (e) {
      setError(String(e));
      setPhase("error");
    }
  }

  if (phase === "idle" || phase === "checking") return null;

  if (phase === "error") {
    return (
      <div className="flex items-center justify-between gap-2 border-b border-yellow-800 bg-yellow-950/40 px-4 py-1.5 text-xs text-yellow-200">
        <span>Update check failed: {error}</span>
        <button
          type="button"
          onClick={doCheck}
          className="rounded border border-yellow-700 px-2 py-0.5 hover:bg-yellow-900/40"
        >
          Retry
        </button>
      </div>
    );
  }

  if (phase === "available" && update) {
    return (
      <div className="flex items-center justify-between gap-2 border-b border-blue-800 bg-blue-950/40 px-4 py-1.5 text-xs text-blue-200">
        <span>
          Update available: v{update.version}
          {update.date ? ` (${update.date})` : ""}
        </span>
        <button
          type="button"
          onClick={install}
          className="rounded border border-blue-700 px-2 py-0.5 hover:bg-blue-900/40"
        >
          Install &amp; restart
        </button>
      </div>
    );
  }

  if (phase === "downloading") {
    const pct =
      progress.total && progress.total > 0
        ? Math.round((progress.downloaded / progress.total) * 100)
        : null;
    return (
      <div className="border-b border-blue-800 bg-blue-950/40 px-4 py-1.5 text-xs text-blue-200">
        Downloading update{pct !== null ? ` … ${pct}%` : "…"}
      </div>
    );
  }

  if (phase === "ready") {
    return (
      <div className="border-b border-blue-800 bg-blue-950/40 px-4 py-1.5 text-xs text-blue-200">
        Update installed, restarting…
      </div>
    );
  }

  return null;
}
