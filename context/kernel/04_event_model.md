# Event Model Deep Dive (Event Store, Projections, Audit, Tamper Evidence)

**Status:** Canonical.

## 1) “Everything important is an event”

Events cover (non-exhaustive):

- workspace/project lifecycle
- board/canvas state changes
- pipeline/stage/gate transitions
- session spawn/fork/attach/detach
- worktree operations
- task lifecycle changes
- tool requests + tool results
- hook decisions
- policy evaluations (refs)
- secret changes (redacted)
- webhook deliveries

## 2) Event store abstraction

The kernel exposes an `EventStore` interface with these operations:

- `append(events...) -> (seq_global_range)`
- `read_from(seq_global_cursor, limit) -> events`
- `read_stream(stream_id, seq_stream_cursor, limit) -> events`
- `subscribe_from(seq_global_cursor) -> realtime stream`

The concrete backend depends on deployment profile.

## 3) Projections (“read models”)

- Projections are rebuilt from the event log.
- Projections power UI queries, search, filters, and dashboards.
- Projections must be idempotent and tolerate reprocessing.

**Rule:** Projections may be dropped and rebuilt without data loss.

## 4) Audit stream (separate, immutable)

Security requires a separate audit stream:

- Audit entries reference core `event_id`s.
- Audit stream is append-only.
- Audit stream is hardened for retention/export.

## 5) Tamper evidence

### 5.1 Hash chaining

- Each event may include `prev_hash` to form a hash chain.
- Hash chaining is required in enterprise deployments.

### 5.2 Signed checkpoints

- Periodic checkpoints sign a range of events (e.g., every N events or T minutes).
- Checkpoints include:
  - range (`from_seq`..`to_seq`)
  - hash of the last event
  - signing metadata

This avoids per-event signing overhead while preserving tamper evidence.

## 6) Durability vs realtime

Realtime delivery can drop or reconnect.

Durability is guaranteed by:

- event store persistence
- client catch-up using `seq_global` cursors

**Guarantee:** At-least-once delivery to clients, idempotent application by event ID/sequence.

See `kernel/transport.md` for transport layer options (SSE, NDJSON, stdio).

## 7) Cluster profile note

In cluster deployments:

- The durable event stream is JetStream.
- Postgres remains the query/projection store.

The event store abstraction supports both.

---

## References

[nats-jetstream]: https://docs.nats.io/nats-concepts/jetstream "JetStream - NATS Docs"
[postgres-listen]: https://www.postgresql.org/docs/current/sql-listen.html "PostgreSQL LISTEN"
