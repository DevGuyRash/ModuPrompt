# PRD — Kernel & Event System

**Status:** Draft (canonical constraints already locked).

## 1) Purpose

Deliver the foundational kernel: command API, event store, deterministic replay, and projections. This is the base for every mode.

## 2) Goals

- Single-writer daemon (`mpd`) owns all state changes.
- Generic wire command envelope with typed SDKs.
- Immutable event log as source of truth.
- Realtime event streaming to clients (WS/SSE) with cursor catch-up.
- Projection system for efficient queries (board snapshots, session trees, task lists, etc.).
- Tamper evidence readiness (hash chain + signed checkpoints) and separate immutable audit stream.

## 3) Non-goals (v1)

- Full multiplayer editing UX
- Cluster message bus (JetStream) implementation
- Complex snapshots/checkpointing (beyond simple boot acceleration)

## 4) Personas

- Desktop power user (single-node)
- Enterprise admin (server)
- Automation/CI user (CLI)

## 5) Functional requirements

### Command API
- Accept commands via local socket/HTTP (profile-dependent).
- Validate payload against schema.
- Reject unknown fields.
- Require idempotency keys for state-changing commands.
- Enforce optimistic concurrency via `expected_version`.

### Event system
- Emit canonical event envelope with:
  - UUIDv7 IDs
  - global and stream sequence numbers
  - schema versioning and upcasters
- Persist every accepted state transition as events.

### Projections
- Derived read models for:
  - workspace/project list
  - session tree and status
  - worktree attach state (writer lock)
  - tasks lifecycle lists
  - board snapshots
- Rebuildable from events.

### Realtime
- Stream events to clients.
- Support reconnect + catch-up from `seq_global` cursor.

### Audit
- Maintain a separate immutable audit stream referencing event IDs.

## 6) Non-functional requirements

- Deterministic replay: same event log ⇒ same state.
- Crash resilience: replay restores state.
- Performance: supports heavy event throughput without blocking UI.
- Security: fail closed on schema violations and policy boundaries.

## 7) Data model (initial)

Minimum entities:

- Workspace, Project
- Event (append-only)
- Projection tables/views
- Idempotency keys table
- Artifact registry (content hash + metadata)
- Audit entry stream

## 8) Key commands (minimum)

- `daemon.ping`
- `workspace.create|open|list`
- `project.create|list`
- `events.subscribe`

## 9) Key events (minimum)

- `workspace.created|opened`
- `project.created`
- `event.appended` (internal)
- `projection.updated` (optional)

## 10) Milestones

1. Daemon starts, migrates DB, creates runtime dirs.
2. Workspace create/open emits events and updates projections.
3. CLI can connect and watch event stream.
4. Deterministic replay rebuilds projections from event log.
5. Separate audit stream writes for privileged actions.

## 11) Risks

- Schema/versioning discipline drift.
- Projection rebuild time without snapshots.
- Cross-platform realtime transport differences.

---

## References

[rfc-9562]: https://www.rfc-editor.org/rfc/rfc9562.html "RFC 9562: Universally Unique IDentifiers (UUIDs)"
[json-schema-spec]: https://json-schema.org/specification "JSON Schema Specification"
[sqlite-wal]: https://sqlite.org/wal.html "Write-Ahead Logging"
