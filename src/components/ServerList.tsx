import type { ServerView } from "../types/server";
import { StatusBadge } from "./StatusBadge";

type Props = {
  servers: ServerView[];
  selectedId?: string;
  onSelect: (id: string) => void;
};

export function ServerList({ servers, selectedId, onSelect }: Props) {
  if (servers.length === 0) {
    return (
      <div className="text-sm text-neutral-500 italic">(no servers)</div>
    );
  }
  return (
    <ul className="space-y-1">
      {servers.map((s) => {
        const active = s.id === selectedId;
        return (
          <li key={s.id}>
            <button
              type="button"
              onClick={() => onSelect(s.id)}
              className={`flex w-full items-center justify-between gap-2 rounded px-2 py-1.5 text-left text-sm hover:bg-neutral-800 ${
                active ? "bg-neutral-800" : ""
              }`}
            >
              <span className="flex min-w-0 flex-col">
                <span className="truncate">{s.name}</span>
                {s.port !== undefined && (
                  <span className="text-[10px] text-neutral-500">
                    :{s.port}
                  </span>
                )}
              </span>
              <StatusBadge status={s.status} />
            </button>
          </li>
        );
      })}
    </ul>
  );
}
