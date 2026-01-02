# Remote Workers (Architecture Reserved Day One)

**Status:** Canonical architecture (not implemented in MVP).

Remote workers allow enterprise deployments to run tasks on separate machines while keeping the daemon as the control plane.

## 1) Goals

- scale execution horizontally
- isolate untrusted workloads from the control plane
- support heterogeneous backends (local/docker/k8s)

## 2) Control model

- The daemon schedules tasks to workers.
- A worker accepts work via a lease.
- The worker executes the deterministic spec under the assigned backend.
- The worker streams logs and returns artifacts/results.
- The daemon appends events and updates projections.

## 3) Protocol shape (conceptual)

- `worker.register(capabilities)`
- `worker.heartbeat(worker_id, leases...)`
- `worker.claim(task_id) -> lease`
- `worker.stream_logs(lease)`
- `worker.complete(lease, result_refs)`

## 4) Security

- Mutually authenticated transport in enterprise deployments.
- Least-privilege credentials for workers.
- Workers cannot mutate state directly; they only return artifacts and results.

## 5) Reliability

- At-least-once delivery with idempotency keys.
- Crash recovery via lease expiration + retry policy.

