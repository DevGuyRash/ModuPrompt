# Roadmap (Phased Delivery)

**Status:** Canonical direction.

This roadmap is intentionally kernel-first.

## Phase 0 — Contracts & PRDs (current)

- Lock invariants, kernel contract, security model, PRDs.

## Phase 1 — MVP: Daemon + Event System + CLI + Orchestrator Core

**Goal:** A runnable headless orchestrator with deterministic state changes.

Deliverables:
- daemon skeleton (`mpd`)
- SQLite event store + projections + migrations
- command API (generic wire + typed Rust SDK)
- realtime event stream (WS/SSE)
- CLI (`mpctl`) with auto-start daemon, JSON output, `events watch`
- workspace/project CRUD
- session spawn/fork + lineage tree
- worktree attach model (single writer lock)
- baseline external CLI runner (PTY capture, timeouts)
- task lifecycle + retries + timeouts + deadlines fields
- minimal board projection for UI and CLI queries

## Phase 2 — Agent Builder v1 (Blueprints + Hooks)

Deliverables:
- blueprint file format + GUI editor (initial)
- hook pipeline with phases + fail-closed behavior
- WASM sandbox hostcalls (`emit_event`, `request_tool_call`, `capsules`, scoped FS)
- hook test harness

## Phase 3 — Tools & Skills v1

Deliverables:
- tool registry with strict JSON schema validation
- progressive disclosure primitives: search/load/run
- skills registry (Agent Skills format) with optional signing
- git tool suite + file ops tools + workflow utilities

## Phase 4 — Enterprise Single-node

Deliverables:
- Postgres profile
- multi-user auth (OIDC/SAML)
- ABAC enforcement end-to-end
- immutable audit stream + signed checkpoints
- server deployment artifacts (docker image + helm chart)
- web UI baseline (orchestrator board)

## Phase 5 — Enterprise Cluster + Execution Backends

Deliverables:
- NATS JetStream profile
- worker protocol (remote workers)
- k8s Jobs backend
- optional Redis cache/coordination

## Phase 6 — Multiplayer / Collaboration

Deliverables:
- server-authoritative collaboration
- shared boards and shared session visibility
- later: live cursors, concurrent edits, conflict policies
- later: CRDT if needed

## Phase 7 — Additional capabilities (post-MVP)

- Context engineering (prompt graphs)
- Web Librarian
- MCP management
- TUI
