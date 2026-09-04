export type SetupIssue = {
  code: string;
  message: string;
};

export type RuntimeStatus = {
  ready: boolean;
  host: string;
  codex_found: boolean;
  codex_path: string | null;
  codex_version: string | null;
  codex_home: string;
  config_toml_exists: boolean;
  auth_json_exists: boolean;
  model: string | null;
  model_provider: string | null;
  azure_endpoint: string | null;
  env_key_name: string | null;
  env_key_present: boolean;
  suggested_workspace: string | null;
  issues: SetupIssue[];
};

export type Chat = {
  id: string;
  title: string;
  created_at: string;
  updated_at: string;
  codex_thread_id: string | null;
};

export type Message = {
  id: string;
  chat_id: string;
  role: "user" | "assistant" | string;
  content: string;
  created_at: string;
  status: "complete" | "running" | "error" | string;
};

export type StreamEvent = {
  chat_id: string;
  message_id: string;
  kind: string;
  text: string;
  thread_id?: string | null;
};

export type Agent = {
  id: string;
  name: string;
  brief: string;
  template: string;
  status: string;
  workspace_path: string | null;
  chat_id: string | null;
  worker_thread_id: string | null;
  grader_thread_id: string | null;
  created_at: string;
  updated_at: string;
};

export type HillclimbRun = {
  id: string;
  agent_id: string;
  goal: string;
  success_criteria: string;
  max_iterations: number;
  current_iteration: number;
  status: string;
  last_grade: string | null;
  last_gaps: string | null;
  allow_writes: boolean;
  created_at: string;
  updated_at: string;
};

export type HillclimbIteration = {
  id: string;
  run_id: string;
  iteration: number;
  phase: string;
  worker_summary: string | null;
  grade: string | null;
  gaps: string | null;
  created_at: string;
};

export type HillclimbEvent = {
  run_id: string;
  agent_id: string;
  kind: string;
  iteration: number;
  phase: string;
  text: string;
  grade?: string | null;
};

export type AuditEvent = {
  id: string;
  at: string;
  action: string;
  actor: string;
  entity_type: string;
  entity_id: string;
  detail: string;
};
