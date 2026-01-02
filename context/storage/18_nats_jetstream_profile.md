# Cluster Profile (NATS JetStream)

**Status:** Canonical.

## 1) Role of JetStream

In cluster deployments, JetStream provides:

- durable event streaming
- replay for many consumers
- fan-out without overloading the projection DB

JetStream is the **durable event transport** in cluster mode.

## 2) Role of Postgres

Postgres remains the query/projection store:

- ABAC policy data
- workspace/project indexes
- board snapshots
- search indexes
- reporting and exports

## 3) Correctness model

- The event stream is durable and replayable.
- Clients may miss realtime messages and must support catch-up.
- Consumers apply events idempotently by event ID/sequence.

## 4) Upgrade path

Deployments can move from single-node (Postgres) to cluster by:

- enabling JetStream
- switching event append/subscription to JetStream
- keeping Postgres projections intact

---

## References

[nats-jetstream]: https://docs.nats.io/nats-concepts/jetstream "JetStream - NATS Docs"
