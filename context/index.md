# Context Docs — Map (Progressive Disclosure)

This directory is the **living specification** of ModuPrompt.

Start from the top and go deeper only as needed.

## 0) Start here

1. `project_overview.md` — refined product overview (vision + scope)
2. `00_invariants.md` — non-negotiables (kernel, determinism, security)
3. `02_architecture_overview.md` — layered architecture (headless-first)

## 1) Kernel & event system

- `03_kernel_contract.md` — command + event envelopes, IDs, ordering, versioning, concurrency
- `kernel/04_event_model.md` — event store, projections, replay, tamper evidence, audit stream

## 2) Orchestration semantics (the MVP)

- `05_orchestration_ontology.md` — boards, pipelines, stages, gates, approvals, tasks, retries/timeouts
- `orchestration/boards.md` — board model (canvas + staging)
- `orchestration/pipelines.md` — pipeline templates → binding → instantiation
- `orchestration/tasks.md` — task lifecycle, retries, deadlines, timeouts
- `orchestration/git_worktrees.md` — hybrid git strategy + writer lock
- `orchestration/webhooks.md` — incoming/outgoing webhooks + triggers

## 3) Agent system

- `agent/06_agent_system.md` — sessions, worktrees, lineage, spawn/fork semantics
- `agent/07_comms_and_channels.md` — scopes, groups, leases, message types
- `agent/08_capsules_and_context.md` — capsules, hard budgets, transcript access

## 4) Agent Builder (Blueprints + hooks + WASM)

- `agent/09_agent_blueprint_spec.md` — file format, version pinning, policy bundle pinning
- `agent/10_hooks.md` — hook phases, trust tiers, failure policy, approvals
- `tooling/11_wasm_sandbox.md` — WASM runtime, capabilities, hostcalls, deterministic logging

## 5) Tools, skills, and optional integrations

- `tooling/12_tools_and_schemas.md` — JSON Schema tools, strict validation, side-effect taxonomy
- `tooling/13_skills.md` — Agent Skills packaging + signing model
- `tooling/14_mcp_strategy.md` — MCP approach (optional, later)

## 6) Storage & deployment profiles

- `storage/15_storage_profiles.md` — local vs enterprise vs cluster composition
- `storage/16_sqlite_profile.md` — WAL + single writer
- `storage/17_postgres_profile.md` — enterprise baseline
- `storage/18_nats_jetstream_profile.md` — cluster streaming

## 7) Security

- `security/19_security_architecture.md` — ABAC+RBAC, policy engine, exfil controls, audit
- `security/20_secrets.md` — envelope encryption, vault UX, redaction strategy
- `security/21_policy_dsl.md` — policy language and evaluation points
- `security/22_audit.md` — immutable audit stream, signing checkpoints

## 8) Execution backends

- `execution/23_backends.md` — local/docker/podman/k8s/jobs
- `execution/24_remote_workers.md` — worker protocol (designed day one)

## 9) UI constraints

- `ui/25_ui_architecture.md` — GPU canvas, LOD, hybrid rendering, daemon separation

## 10) Roadmap

- `roadmap.md` — phased delivery

## 11) Validation

- `validation/issue_002_hello_world.md` — hello-world kernel validation (Issue #2)

## 12) Future capabilities (captured vision)

- `future/context_engineering_mode.md` — prompt graph + diagram objects + templating syntax

## PRDs

All PRDs live in `context/prd/`:

- Kernel & Event System
- Orchestrator Mode
- Agent Builder
- Tools/Skills/MCP Strategy
- Security & Enterprise
- Web Librarian
- Multiplayer/Collab
- CLI/TUI
- Storage & Deployment Profiles

## Reference registry

- `references.md` — canonical link registry used across docs
