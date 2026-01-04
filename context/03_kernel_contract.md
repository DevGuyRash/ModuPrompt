# Kernel Contract (Commands, Events, Ordering, Replay)

**Status:** Canonical.

This document defines the stable kernel contract that all clients (GUI, web, CLI, SDKs) must follow.

## 1) Command model

### 1.1 Command envelope (wire)

Commands are **typed in SDKs** but use a **generic envelope on the wire**.

#### Required fields

- `type`: string (namespaced, e.g., `workspace.create`)
- `schema_version`: integer (command payload schema)
- `payload`: JSON object (must validate against schema)
- `idempotency_key`: string (required for all state-changing commands)
- `trace_id`: string (correlation across command → events)

#### Optional fields

- `expected_version`: integer (optimistic concurrency guard)
- `actor_override`: (internal only; never exposed to untrusted clients)

#### Example

```json
{
  "type": "project.create",
  "schema_version": 1,
  "idempotency_key": "ik_01HQ...",
  "expected_version": 42,
  "trace_id": "tr_01HQ...",
  "payload": {
    "workspace_id": "0190f8e2-...",
    "name": "core"
  }
}
```

### 1.2 Command processing rules (normative)

- The daemon MUST validate command payloads against their schema.
- The daemon MUST reject unknown fields (fail closed).
- For state-changing commands, the daemon MUST enforce idempotency using `idempotency_key`.
- If `expected_version` is provided and does not match the current version, the daemon MUST reject the command.
- Command handling MUST be serialized inside the daemon for a given workspace.

## 2) Event model

### 2.1 Event envelope (canonical)

Events are immutable records emitted by the daemon.

#### Required baseline fields

- `event_id`: UUIDv7
- `event_type`: string (namespaced)
- `timestamp`: RFC3339 timestamp
- `actor`: identity reference (user/session/system)
- `workspace_id`
- `project_id` (optional)
- `subject`: `{ kind, id }` (primary entity affected)
- `payload`: JSON object (schema-defined)
- `schema_version`: integer
- `seq_global`: integer (monotonic per workspace)
- `seq_stream`: integer (monotonic per stream)

#### Optional fields (enterprise/cluster)

- `prev_hash`: hash of previous event (hash-chain)
- `sig`: signature (checkpointing / signed snapshots)
- `trace_id`: copied from command when applicable
- `causality`: `{ command_id?, parent_event_id?, root_event_id? }`
- `policy_decision_ref`: reference to policy evaluation/audit entry

### 2.2 Ordering guarantees

- `seq_global` defines a single total order per workspace.
- `seq_stream` defines an order for a specific stream (e.g., project stream, board stream, session stream).
- Consumers MUST treat `event_id` + `seq_*` as the idempotency key for application.

### 2.3 Event versioning

- Payloads SHOULD be additive when possible.
- The daemon MUST support upcasters from older schema versions to the current representation.

## 3) Replay + projections

### 3.1 Source of truth

- The event log is the source of truth.
- Projections are derived and may be rebuilt.

### 3.2 Default replay policy

- Default: full rebuild (events → projections) for correctness.
- Later: snapshots/checkpoints may accelerate boot.

## 4) IDs and artifacts

- Entity/event IDs use UUIDv7.
- Artifacts are content-addressed via `blake3(content)` and stored with metadata.

## 5) Conflict handling

- Conflicts are explicit.
- No last-write-wins.
- Rejections MUST return machine-readable error codes and SHOULD emit a rejection event for audit.

### 5.1 Error codes (canonical)

The daemon MUST return a structured error payload on failure (HTTP non-2xx responses and stdio `error` frames):

```json
{
  "code": "invalid_schema",
  "message": "human-readable summary",
  "details": {},
  "trace_id": "tr_..."
}
```

`details` and `trace_id` are optional. `code` MUST be one of the following canonical values:

See `schemas/transport/error_response.v1.json` for the canonical schema.

- `invalid_schema` — payload failed JSON schema validation
- `unknown_command` — command type not recognized
- `idempotency_key_required` — missing idempotency key for state-changing command
- `expected_version_mismatch` — optimistic concurrency check failed
- `validation_failed` — semantic validation failed
- `unauthorized` — auth missing or invalid
- `not_found` — resource not found
- `policy_denied` — denied by policy or safety mode
- `unknown` — unclassified error
- `internal` — reserved for internal server errors

Rejections emitted as `command.rejected` MUST use the same `code` set.

## 6) Realtime streaming

- The daemon exposes event streams to clients via multiple transports (SSE, NDJSON, stdio).
- Realtime is best-effort; durability comes from the event store.
- Clients MUST be able to resync from the event log using sequence cursors.

See `kernel/transport.md` for transport layer details.

---

## References

[rfc-9562]: https://www.rfc-editor.org/rfc/rfc9562.html "RFC 9562: Universally Unique IDentifiers (UUIDs)"
[json-schema-spec]: https://json-schema.org/specification "JSON Schema Specification"
[blake3]: https://github.com/BLAKE3-team/BLAKE3 "BLAKE3 official repo"
