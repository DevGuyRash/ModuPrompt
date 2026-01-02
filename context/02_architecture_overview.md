# Architecture Overview (Headless-First, Layered)

**Status:** Canonical.

ModuPrompt is built as a layered system so that **the GUI is optional** and the entire product can run headless.

## 1) Architectural layers

```
Layer 4: Interfaces
  • Desktop GUI (GPU canvas)
  • Web GUI (enterprise)
  • CLI (CI/CD)
  • (Later) TUI

Layer 3: Client SDKs
  • Typed SDKs (Rust/TS/...) built on a generic wire contract
  • Backpressure-aware streaming event subscriptions

Layer 2: CLI
  • Composable shell commands
  • JSON output, stable exit codes

Layer 1: Daemon API + Realtime
  • Command API: propose actions and request reads
  • Realtime: subscribe to event streams (WS/SSE)
  • Access control + policy enforcement

Layer 0: Kernel
  • Core primitives: workspaces, projects, boards, sessions, worktrees, tasks
  • Event store (source of truth)
  • Projections (read models)
  • Tool registry + deterministic execution
  • Hooks/policy pipeline (WASM)
```

## 2) Process model

### Default local profile (desktop)

- `mpd` (daemon) is the **only writer**.
- `mpctl` (CLI) and the GUI connect to the daemon and never write storage directly.
- Daemon uses an embedded DB (SQLite) and exposes realtime streams.

### Enterprise profile (server)

- Daemon runs as a server process with multi-user auth and ABAC.
- Storage is Postgres.
- Web UI connects to the daemon; desktop UI may connect remotely.

### Cluster profile

- Event transport is upgraded to NATS JetStream.
- Postgres remains the projection/query store.

## 3) “Deterministic action kernel”

Kernel invariants:

- Agents do not mutate state directly.
- Agents propose actions.
- The kernel gates by policy/approvals.
- The kernel executes deterministic tools.

This is the foundation for:

- auditability
- replayability
- webhooks
- multiplayer

## 4) Progressive disclosure as a first-class design

To prevent context bloat:

- tools and skills are discovered via search
- only selected definitions are loaded
- tool results are summarized into capsules

## 5) Implementation constraints

- Rust-first across daemon, CLI, and UI.
- Desktop profile avoids required runtimes (Node/Python/JVM).
- Extensions/hook logic runs in a sandboxed WASM runtime.

---

## References

[sqlite-when-to-use]: https://sqlite.org/whentouse.html "Appropriate Uses For SQLite"
[nats-jetstream]: https://docs.nats.io/nats-concepts/jetstream "JetStream - NATS Docs"
