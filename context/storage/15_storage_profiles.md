# Storage & Deployment Profiles (Composition)

**Status:** Canonical.

ModuPrompt supports multiple deployment profiles without rewriting the kernel.

## 1) Profile: Desktop / Local-first (default)

**Goals:** zero required runtime deps, portability, single-user performance.

- Event store: SQLite event table
- Projections/search: SQLite tables (FTS optional)
- Realtime: daemon websocket/SSE
- Event bus: in-process channels
- Cache: in-memory

**Rule:** daemon is the only writer (single-writer pattern)

## 2) Profile: Enterprise Single-node

**Goals:** multi-user concurrency, centralized ops, policy, audit.

- Event store: Postgres event table
- Projections/query: Postgres
- Realtime: WebSocket/SSE
- Delivery: tailing/polling + optional LISTEN/NOTIFY wakeups
- Minimal external deps: Postgres only

## 3) Profile: Enterprise Cluster

**Goals:** scale, durable streaming, replay for many consumers.

- Event transport/store: NATS JetStream (durable stream + replay)
- Projections/query: Postgres
- Cache/coordination: optional Redis

## 4) Cross-profile invariants

Across all profiles:

- event log is the source of truth
- realtime is best-effort; clients catch up from the log
- audit is separate immutable stream
- tool execution is deterministic and policy-gated

---

## References

[sqlite-when-to-use]: https://sqlite.org/whentouse.html "Appropriate Uses For SQLite"
[sqlite-wal]: https://sqlite.org/wal.html "Write-Ahead Logging"
[postgres-listen]: https://www.postgresql.org/docs/current/sql-listen.html "PostgreSQL LISTEN"
[nats-jetstream]: https://docs.nats.io/nats-concepts/jetstream "JetStream - NATS Docs"
[redis-pubsub]: https://redis.io/docs/latest/develop/pubsub/ "Redis Pub/Sub"
