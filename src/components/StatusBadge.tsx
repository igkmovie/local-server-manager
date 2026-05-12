import type { ServerStatus } from "../types/server";

const STYLES: Record<ServerStatus, string> = {
  stopped: "bg-neutral-700 text-neutral-300",
  starting: "bg-blue-600 text-white",
  running: "bg-green-600 text-white",
  unhealthy: "bg-yellow-500 text-black",
  stopping: "bg-blue-400 text-black",
  error: "bg-red-600 text-white",
  external: "bg-purple-600 text-white",
};

export function StatusBadge({ status }: { status: ServerStatus }) {
  return (
    <span
      className={`inline-block rounded px-1.5 py-0.5 text-[10px] font-medium uppercase tracking-wider ${STYLES[status]}`}
    >
      {status}
    </span>
  );
}
