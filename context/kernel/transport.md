# Daemon Transport Layer

**Status:** Canonical.

This document describes the transport options for daemon API communication.

## 1) Overview

The daemon exposes its API over multiple transports. All transports share the same command/event envelope schema and semantics—only the wire format differs.

| Transport | Endpoint / Mode | Use Case |
|-----------|-----------------|----------|
| SSE | `GET /v1/events/stream` | Browser clients, JS SDKs |
| NDJSON | `GET /v1/events/stream-ndjson` | CLI tools, scripts, non-browser clients |
| Stdio | `mpd serve-stdio` | Embedded daemon, subprocess integration |

## 2) SSE (Server-Sent Events)

**Default transport.**

- Standard `text/event-stream` format.
- Each event is prefixed with `data:` followed by JSON.
- Supports `workspace_id` and `from` cursor params.
- Browser-native via `EventSource`.

Request:

```
GET /v1/events/stream?workspace_id=<id>&from=<seq>
```

Response:

```
data: {"event_id":"ev_...","event_type":"workspace.created",...}

data: {"event_id":"ev_...","event_type":"project.created",...}
```

## 3) NDJSON (Newline-Delimited JSON)

- One complete JSON object per line.
- `Content-Type: application/x-ndjson`.
- Chunked transfer encoding.
- Easier to parse in shell scripts (`jq`, line-by-line readers).

Request:

```
GET /v1/events/stream-ndjson?workspace_id=<id>&from=<seq>
```

Response:

```
{"event_id":"ev_...","event_type":"workspace.created",...}
{"event_id":"ev_...","event_type":"project.created",...}
```

## 4) Stdio transport

For embedding the daemon as a subprocess without network overhead.

### 4.1 Invocation

```bash
mpd serve-stdio --auth none --db /path/to/db.sqlite
```

Or with token auth:

```bash
mpd serve-stdio --auth token --token <secret>
```

### 4.2 Frame envelope

All stdio communication uses NDJSON frames.

Request frame:

```json
{
  "request_id": "rq_...",
  "type": "command.submit",
  "schema_version": 1,
  "payload": { ... }
}
```

Response frame:

```json
{
  "request_id": "rq_...",
  "type": "command.response",
  "schema_version": 1,
  "payload": { ... }
}
```

Error frame:

```json
{
  "request_id": "rq_...",
  "type": "error",
  "schema_version": 1,
  "payload": {
    "code": "invalid_schema",
    "message": "..."
  }
}
```

Event frame (streamed after `events.subscribe`):

```json
{
  "type": "events.event",
  "schema_version": 1,
  "payload": { ...EventEnvelope... }
}
```

### 4.3 Supported frame types

| Frame Type | Direction | Description |
|------------|-----------|-------------|
| `auth` | request | Authenticate (when `--auth token`) |
| `command.submit` | request | Submit a command envelope |
| `query.workspaces` | request | List workspaces |
| `query.projects` | request | List projects (requires `workspace_id` in payload) |
| `events.subscribe` | request | Subscribe to event stream |
| `*.response` | response | Success response to request |
| `error` | response | Error response |
| `events.event` | push | Streamed event after subscription |

### 4.4 Error codes

| Code | Meaning |
|------|---------|
| `invalid_schema` | Malformed frame or unknown fields |
| `unauthorized` | Auth required or invalid token |
| `unknown_command` | Unknown frame type |
| `conflict` | Subscription already active |
| `internal` | Server error |

## 5) Security

### 5.1 HTTP transports (SSE, NDJSON)

- MUST include `Authorization: Bearer <token>` header.
- Token is generated per daemon session and stored in `$RUNTIME_DIR/daemon.json`.
- Runtime directory and token file have restricted permissions (0700/0600 on Unix).

### 5.2 Stdio transport

- When `--auth none`: No authentication required. Use ONLY for trusted subprocess scenarios where the parent process controls access.
- When `--auth token`: Requires an `auth` frame with valid token before any other operation.
- The stdio transport MUST enforce the same schema validation as HTTP (unknown fields rejected, fail closed).

### 5.3 General

- All transports enforce `#[serde(deny_unknown_fields)]` on envelopes.
- Invalid schema → request rejected with error.
- Policy/ABAC applies identically across transports.

## 6) When to use each transport

| Scenario | Recommended Transport |
|----------|----------------------|
| Browser-based UI | SSE |
| CLI tools (`mpctl`) | SSE (default), NDJSON (simpler parsing) |
| Shell scripts | NDJSON (pipe to `jq`) |
| Embedded daemon in another process | Stdio |
| IDE plugins, editor extensions | Stdio (subprocess) |
| Testing/CI | NDJSON or Stdio |

## 7) CLI examples

```bash
# SSE (default)
mpctl events watch --workspace demo --from 0

# NDJSON
mpctl events watch --workspace demo --from 0 --transport ndjson

# Stdio (spawns mpd serve-stdio internally)
mpctl events watch --workspace demo --from 0 --transport stdio
```

## 8) Schema references

- Stdio frame schema: `schemas/transport/stdio.frame.v1.json`
- Command envelope: `schemas/commands/*.json`
- Event envelope: `schemas/events/*.json`

---

## References

[rfc-8895]: https://www.rfc-editor.org/rfc/rfc8895.html "RFC 8895: Server-Sent Events"
[ndjson-spec]: https://github.com/ndjson/ndjson-spec "NDJSON Specification"
