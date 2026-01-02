# PRD - Storage and Deployment Profiles

**Status:** Draft.

## 1) Purpose

Provide storage and deployment compositions that preserve the same kernel semantics across desktop, enterprise single-node, and enterprise cluster.

## 2) Goals

- Desktop local-first uses SQLite with WAL and a single-writer daemon.
- Enterprise single-node uses Postgres and requires no other services.
- Enterprise cluster adds NATS JetStream for durable event streaming.
- Redis is optional for caching/coordination, never the system of record.
- Workspace portability remains possible via export/import bundles (event log + artifacts + manifest).

## 3) Non-goals (v1)

- Automatic cluster provisioning
- Complex multi-region replication strategies

## 4) Functional requirements

### Desktop
- Single binary runs daemon + clients.
- SQLite migrations run on daemon startup.

### Enterprise single-node
- Postgres schema and migrations supported.
- Auth and ABAC enforced by daemon.

### Enterprise cluster
- Event transport through JetStream.
- Workers architecture reserved and compatible.

### Distribution
- Desktop: installers acceptable; avoid required runtimes.
- Server: docker image + helm chart.

## 5) Non-functional requirements

- Deterministic replay across all profiles.
- Realtime is best-effort; catch-up from event log.
- Safe migrations with backup and recovery.

## 6) Milestones

1. SQLite profile implemented and tested.
2. Postgres profile implemented and tested.
3. JetStream profile designed and prototyped.
4. Docker image + helm chart baseline.

---

## References

[sqlite-wal]: https://sqlite.org/wal.html "Write-Ahead Logging"
[postgres-listen]: https://www.postgresql.org/docs/current/sql-listen.html "PostgreSQL LISTEN"
[nats-jetstream]: https://docs.nats.io/nats-concepts/jetstream "JetStream - NATS Docs"
