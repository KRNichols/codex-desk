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
  store_encrypted?: boolean;
  key_backend?: string | null;
  audit_chain_ok?: boolean;
  session_user?: string;
  machine_bound?: boolean;
  machine_binding_ok?: boolean;
  operator_attested?: boolean;
  pat_slot?: string;
  hello_bind?: string;
  runner_allowlist?: string;
  shared_provider_auth?: boolean;
  shared_auth_note?: string;
  global_agents_md?: boolean;
  global_agents_override_md?: boolean;
  config_has_developer_instructions?: boolean;
  agent_jobs_override?: string;
  issues: SetupIssue[];
};

export type OperatorAttestation = {
  configured: boolean;
  operator_name: string | null;
  organization: string | null;
  statement: string | null;
  at: string | null;
};

export type IdentityStatus = {
  session_user: string;
  machine_id_present: boolean;
  machine_bound: boolean;
  machine_binding_ok: boolean;
  key_backend: string;
  store_encrypted: boolean;
  audit_chain_ok: boolean;
  operator_attestation: OperatorAttestation;
  pat_slot: string;
  hello_bind: string;
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

export type HarnessJob = {
  name: string;
  label: string;
  status: string;
  summary: string;
};

export type HarnessPromotion = {
  id: string;
  category: string;
  gap: string;
  patch: string;
  status: string;
  created_at: string;
  promoted_at: string | null;
};

export type HarnessRecord = {
  jobs: HarnessJob[];
  autonomy_tier: string;
  autonomy_label: string;
  approval_status: string;
  approval_evidence: string | null;
  classified_gap: string | null;
  gap_category: string | null;
  recovery_phase: string;
  promotions: HarnessPromotion[];
  sandbox: string;
  allowlist: string;
};

export type HarnessMap = {
  promotions: HarnessPromotion[];
  notes: string[];
  updated_at: string;
};

export type EnvVarRow = {
  key: string;
  kind: string;
  description: string;
  status: string;
  source: string;
  required: boolean;
  from_config: boolean;
  related_to: string | null;
  display_value: string | null;
  settable: boolean;
};

export type ConfigFieldRow = {
  key: string;
  description: string;
  status: string;
  display_value: string | null;
};

export type SetupEnvStatus = {
  codex_home: string;
  config_path: string;
  config_toml_exists: boolean;
  home_source: string;
  model: string | null;
  model_provider: string | null;
  base_url: string | null;
  env_keys_in_config: string[];
  vars: EnvVarRow[];
  config_fields: ConfigFieldRow[];
  note: string;
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
  harness?: HarnessRecord;
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
  prev_hash?: string;
  event_hash?: string;
};
