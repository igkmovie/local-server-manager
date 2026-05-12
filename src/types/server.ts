export type ServerStatus =
  | "stopped"
  | "starting"
  | "running"
  | "unhealthy"
  | "stopping"
  | "error"
  | "external";

export type ServerView = {
  id: string;
  name: string;
  status: ServerStatus;
  port?: number;
  healthUrl?: string;
  openUrl?: string;
  command: string;
  args: string[];
  cwd: string;
  pid?: number;
  lastStartedAt?: string;
  lastStoppedAt?: string;
  lastExitCode?: number;
  lastError?: string;
};

export type GroupView = {
  id: string;
  name: string;
  serverIds: string[];
};
