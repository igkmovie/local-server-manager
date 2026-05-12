import type { GroupView } from "../types/server";

type Props = {
  groups: GroupView[];
};

export function GroupList({ groups }: Props) {
  if (groups.length === 0) {
    return <div className="text-sm text-neutral-500 italic">(no groups)</div>;
  }
  return (
    <ul className="space-y-1">
      {groups.map((g) => (
        <li
          key={g.id}
          className="rounded px-2 py-1.5 text-sm text-neutral-300"
        >
          <div className="truncate">{g.name}</div>
          <div className="text-[10px] text-neutral-500">
            {g.serverIds.length} servers
          </div>
        </li>
      ))}
    </ul>
  );
}
