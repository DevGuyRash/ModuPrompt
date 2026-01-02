# Enterprise Profile (Postgres)

**Status:** Canonical.

## 1) Why Postgres for enterprise

Postgres provides multi-user concurrency, operational tooling, and centralized durability suitable for server deployments.

## 2) Baseline requirement

Single-node enterprise requires only:

- daemon
- Postgres

No additional bus/cache is required.

## 3) Realtime

Realtime delivery is best-effort; correctness comes from the event log.

Options:

- WebSocket/SSE streaming of newly appended events
- DB-backed tailing (clients resync by sequence cursors)
- LISTEN/NOTIFY for wakeups (not durable) coupled with tailing for catch-up

## 4) Multi-tenancy posture

- ABAC is enforced by the daemon policy engine.
- DB roles are defensive-in-depth but do not replace the daemon.

---

## References

[postgres-listen]: https://www.postgresql.org/docs/current/sql-listen.html "PostgreSQL LISTEN"
