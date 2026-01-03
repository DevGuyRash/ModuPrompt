# ModuPrompt

ModuPrompt is a **deterministic orchestration control plane for agents**.

It is **not** “chat with an LLM.” Agents are first-class entities with controlled capabilities, deterministic execution semantics, lineage-scoped communication, and an event-sourced kernel that can run **headless**.

The UI (desktop or web) is *an interface*, not the product.

## What you get (the 5-minute wow)

- A living “orchestration universe”: multiple projects; sessions with lineage; worktrees; tasks; gates; approvals.
- Agents can spawn other agents and coordinate via **lineage-scoped channels**.
- A deterministic pipeline model (stages/gates/approvals) where **state changes are tool-executed and replayable**.
- “Agent Builder”: agents are assembled as **pipelines of hooks/middleware** (pre-send, post-receive, policy checks, tool routing), not a single prompt blob.

## Core principles (non-negotiables)

- **Kernel is the product**: single-writer daemon owns all writes; everything else is a client.
- **Event log is the source of truth**: projections are derived; replay is deterministic.
- **Strict tool schema**: tools are invoked via validated schemas only; fail closed.
- **Propose → Gate → Execute**: agents propose; kernel gates by policy; kernel executes deterministically.
- **Progressive disclosure**: search → load → run for tools/skills/context; hard budgets enforced by kernel.
- **Security by default**: ABAC+RBAC, deny-by-default network, encrypted secrets at rest, immutable audit stream.
- **Programmable but safe**: hooks/extensions run in a sandboxed **WASM runtime** by default; hostcalls are capability-scoped and logged.
- **Rust-first**: the chain is designed to be almost entirely Rust.

See: `context/00_invariants.md`.

## Architecture (layered, headless-first)

The system is intentionally layered so the GUI is optional:

- **Layer 4 — Interfaces**: Desktop GUI, Web GUI, CLI, (future) TUI
- **Layer 3 — Typed Client SDKs**: fully typed APIs for Rust/TS/etc. (wire is generic)
- **Layer 2 — CLI**: composable commands for CI/CD and automation
- **Layer 1 — API**: daemon exposes command API + realtime event streams
- **Layer 0 — Core primitives**: workspaces/projects/boards/sessions/worktrees/tasks/policies/events

See: `context/02_architecture_overview.md`.

## Deployment profiles

ModuPrompt supports different profiles without rewriting the kernel:

- **Desktop / Local-first (default)**
  - Storage: SQLite (WAL) with a single-writer daemon
  - Event bus: in-process + daemon websocket to clients
  - Runtime deps: none required beyond OS components; optional integrations (git, docker/podman, agent CLIs)

- **Enterprise (single-node team server)**
  - Storage: Postgres
  - Realtime: WebSocket/SSE
  - Minimal external deps: Postgres only

- **Enterprise Cluster (scale & replay)**
  - Event transport: NATS JetStream (durable stream + replay)
  - Storage/projections: Postgres
  - Optional: Redis for caching/coordination

See: `context/storage/15_storage_profiles.md`.

## Repository status

This repository is **docs-first** right now: the kernel contracts, security model, and PRDs are specified before scaffolding code.

Implementation planning lives in:

- `context/roadmap.md` — phased delivery goals

## Project hygiene (labels + milestones)

We use a consistent issue taxonomy:

- Type: `type/epic`, `type/feature`, `type/bug`, `type/chore`, `type/spike`, `type/security`
- Priority: `P0`, `P1`, `P2`
- Area: `area/*` (full list in `gh_issues.md`)

Milestones track the delivery phases (Phase 1 through Phase 7). See `gh_issues.md` for the current phase naming.

## Quickstart (target interface)

These commands describe the intended developer UX (exact binaries will be implemented as the repo scaffolds):

```bash
# Build everything
cargo build --workspace

# Start the daemon (auto-migrates local DB, creates runtime dirs)
mpd start

# Initialize a workspace
mpctl workspace init ./my-workspace

# Watch the event stream (server-sent events or websocket)
mpctl events watch --workspace ./my-workspace --json

# Create a project
mpctl project create --workspace ./my-workspace --name core

# Spawn a session (agent adapter) + optionally a worktree
mpctl session spawn --project core --blueprint ./agents/blueprints/reviewer.yaml
```

## Docs map

Start here:

- `context/index.md` — documentation map (progressive disclosure)
- `context/project_overview.md` — refined product overview
- `context/prd/` — PRDs (mode + cross-cutting)

---

## License

- Core: intended to be open-source.
- Enterprise: intended to be commercial.

Final licensing will be decided and documented explicitly.
