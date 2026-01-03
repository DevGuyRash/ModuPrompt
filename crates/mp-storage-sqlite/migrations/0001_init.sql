PRAGMA journal_mode=WAL;
PRAGMA foreign_keys=ON;

CREATE TABLE IF NOT EXISTS schema_migrations (
  version INTEGER PRIMARY KEY,
  applied_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS events (
  workspace_id TEXT NOT NULL,
  seq_global INTEGER NOT NULL,
  stream_id TEXT NOT NULL,
  seq_stream INTEGER NOT NULL,
  event_id TEXT NOT NULL,
  event_type TEXT NOT NULL,
  ts TEXT NOT NULL,
  actor_json TEXT NOT NULL,
  project_id TEXT,
  subject_kind TEXT NOT NULL,
  subject_id TEXT NOT NULL,
  schema_version INTEGER NOT NULL,
  payload_json TEXT NOT NULL,
  trace_id TEXT,
  PRIMARY KEY (workspace_id, seq_global),
  UNIQUE (event_id),
  UNIQUE (workspace_id, stream_id, seq_stream)
);

CREATE INDEX IF NOT EXISTS idx_events_workspace_seq
  ON events (workspace_id, seq_global);
CREATE INDEX IF NOT EXISTS idx_events_stream_seq
  ON events (workspace_id, stream_id, seq_stream);

CREATE TABLE IF NOT EXISTS idempotency_keys (
  workspace_id TEXT NOT NULL,
  idempotency_key TEXT NOT NULL,
  command_type TEXT NOT NULL,
  trace_id TEXT NOT NULL,
  first_seq_global INTEGER NOT NULL,
  last_seq_global INTEGER NOT NULL,
  status_code TEXT NOT NULL,
  PRIMARY KEY (idempotency_key, command_type)
);

CREATE INDEX IF NOT EXISTS idx_idempotency_workspace
  ON idempotency_keys (workspace_id);

CREATE TABLE IF NOT EXISTS proj_workspaces (
  workspace_id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  root_path TEXT NOT NULL,
  created_at TEXT NOT NULL,
  seq_global INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS proj_projects (
  project_id TEXT PRIMARY KEY,
  workspace_id TEXT NOT NULL,
  name TEXT NOT NULL,
  created_at TEXT NOT NULL,
  seq_global INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS proj_meta (
  workspace_id TEXT PRIMARY KEY,
  last_seq_global INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS artifacts (
  hash TEXT PRIMARY KEY,
  size_bytes INTEGER NOT NULL,
  mime_type TEXT,
  created_at TEXT NOT NULL,
  metadata_json TEXT NOT NULL
);
