# Execution Backends (Local, Docker, Podman, Kubernetes Jobs)

**Status:** Canonical.

Execution backends provide isolation and resource control for tasks, tools, and agent runtimes.

## 1) Backend abstraction

The kernel defines an `ExecutionBackend` trait (conceptually):

- `spawn_process(spec) -> handle`
- `stream_logs(handle) -> stream`
- `signal(handle, kind)`
- `wait(handle) -> exit_status`
- `enforce_limits(handle, cpu/mem/time)`

Backends must integrate with:

- policy engine (who can run what)
- secret injection rules
- filesystem scoping (worktree allowlists)
- audit logging

## 2) Local process backend (default)

- Used unless a user opts into containers.
- Uses OS process primitives; captures PTY output when possible.
- Enforces timeouts and resource limits where supported.

## 3) Docker backend (optional)

- Uses host docker via a **socket proxy** (recommended) rather than Docker-in-Docker.
- Container execution is policy-gated and audited.

## 4) Podman backend (optional)

- Separate backend implementation.
- Uses docker-compatible socket where possible.

## 5) Kubernetes backend (later)

First integration runs tasks as **Kubernetes Jobs**.

- Jobs are created with strict resource limits and security context.
- Logs and artifacts are streamed and stored.

## 6) Resource limits (required)

All tasks must have CPU/memory/time budgets with sane defaults.
Server-side enforcement is mandatory.
