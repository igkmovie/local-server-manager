import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ServerList } from "./components/ServerList";
import { GroupList } from "./components/GroupList";
import { ServerDetail } from "./components/ServerDetail";
import { ConfigErrorBanner } from "./components/ConfigErrorBanner";
import { LogViewer } from "./components/LogViewer";
import { UpdateBanner } from "./components/UpdateBanner";
import type { GroupView, ServerView } from "./types/server";
import "./App.css";

function App() {
  const [servers, setServers] = useState<ServerView[]>([]);
  const [groups, setGroups] = useState<GroupView[]>([]);
  const [selectedId, setSelectedId] = useState<string | undefined>();
  const [configError, setConfigError] = useState<string | null>(null);

  async function load() {
    try {
      const [srvs, grps] = await Promise.all([
        invoke<ServerView[]>("get_servers"),
        invoke<GroupView[]>("get_groups"),
      ]);
      setServers(srvs);
      setGroups(grps);
      setConfigError(null);
      setSelectedId((cur) => cur ?? srvs[0]?.id);
    } catch (e) {
      setConfigError(String(e));
    }
  }

  useEffect(() => {
    load();
    const id = setInterval(load, 2000);
    return () => clearInterval(id);
  }, []);

  async function revealConfig() {
    try {
      await invoke("reveal_config_file");
    } catch (e) {
      console.error(e);
    }
  }

  const selected = servers.find((s) => s.id === selectedId);

  return (
    <div className="flex h-screen flex-col bg-neutral-950 text-neutral-100">
      <header className="flex items-center justify-between border-b border-neutral-800 px-4 py-2">
        <h1 className="text-sm font-semibold tracking-wide">
          Local Server Manager
        </h1>
        <div className="flex gap-2">
          <button
            type="button"
            onClick={load}
            className="rounded border border-neutral-700 px-2 py-1 text-xs hover:bg-neutral-800"
          >
            Reload
          </button>
          <button
            type="button"
            onClick={revealConfig}
            className="rounded border border-neutral-700 px-2 py-1 text-xs hover:bg-neutral-800"
          >
            Open config
          </button>
        </div>
      </header>

      <UpdateBanner />

      {configError && (
        <ConfigErrorBanner
          message={configError}
          onReveal={revealConfig}
          onRetry={load}
        />
      )}

      <div className="flex flex-1 overflow-hidden">
        <aside className="w-64 shrink-0 overflow-y-auto border-r border-neutral-800 p-2">
          <div className="mb-2 text-xs uppercase tracking-wider text-neutral-400">
            Servers
          </div>
          <ServerList
            servers={servers}
            selectedId={selectedId}
            onSelect={setSelectedId}
          />
          <div className="mt-4 mb-2 text-xs uppercase tracking-wider text-neutral-400">
            Groups
          </div>
          <GroupList groups={groups} onChanged={load} />
        </aside>

        <main className="flex flex-1 flex-col overflow-hidden">
          <section className="border-b border-neutral-800 p-4">
            <div className="mb-2 text-xs uppercase tracking-wider text-neutral-400">
              Detail
            </div>
            <ServerDetail server={selected} onChanged={load} />
          </section>
          <section className="flex flex-1 flex-col overflow-hidden p-4">
            <div className="mb-2 text-xs uppercase tracking-wider text-neutral-400">
              Logs
            </div>
            <div className="min-h-0 flex-1">
              <LogViewer serverId={selectedId} />
            </div>
          </section>
        </main>
      </div>
    </div>
  );
}

export default App;
