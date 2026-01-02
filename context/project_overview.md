# ModuPrompt — Project Overview (Refined)

**Status:** Canonical product overview.

## 1) One sentence

ModuPrompt is a **local-first, event-sourced orchestration kernel** that makes agents first-class citizens: they run in controlled execution environments, communicate through policy-enforced lineage scopes, and advance through deterministic pipelines that are inspectable and replayable.

## 2) What ModuPrompt is (and isn’t)

### It is

- A **control plane** for agentic work: sessions, worktrees, tasks, artifacts, gates, approvals.
- A way to build “agents” as **hook/middleware pipelines** (not a single system prompt blob).
- A system where **state changes are tool-executed and logged** (Propose → Gate → Execute).
- A platform that can run **headless** (daemon + API), with multiple clients (GUI/web/CLI).

### It is not

- Not a generic “chat app with tools.”
- Not a full IDE replacement.
- Not a Git hosting product.
- Not a magical nondeterministic automation engine (actions must be inspectable and replayable).

## 3) Primary users

### Day 1

- Solo power users and senior engineers/team leads who already juggle multiple repos + multiple LLM tools and want a deterministic, auditable system to manage the chaos.
- Platform/DevOps-minded builders who care about policy, automation, security, and repeatability.

### Later

- Teams and enterprises needing multiplayer collaboration, RBAC/ABAC policy, auditability, secrets handling, and safe execution.

## 4) Product pillars

1. **Determinism you can trust**
   - Same event log ⇒ same state.
   - Nondeterministic outputs become artifacts that are pinned and replayable.

2. **Headless kernel, optional UI**
   - Daemon owns writes.
   - GUI is a client on top of stable API + event stream.

3. **Security-first (enterprise-compatible from day one)**
   - ABAC+RBAC, capability tokens, deny-by-default network.
   - Secrets encrypted at rest, redacted derivatives for search/index.
   - Immutable audit stream, tamper-evident event chain.

4. **Progressive disclosure at scale**
   - Tools/skills/context are discovered and loaded on-demand.
   - Hard budgets enforced by the kernel.

5. **Programmable but safe**
   - Hooks/extensions run in sandboxed WASM.
   - Deterministic hostcalls are logged.

## 5) The 5-minute “wow”

Open the app and immediately see a living orchestration universe:

- Multiple projects.
- Sessions organized into a **session tree** (lineage).
- Worktree-per-session where appropriate (with 1 writer and many readers).
- Agents can spawn agents and exchange messages in **lineage-scoped channels**.
- A deterministic pipeline where work advances through stages, gates, and approvals.

This is **governable agentic work**, not chat.

## 6) Core primitives (cross-mode)

- Workspace, Project
- Board/Canvas
- Session, Agent
- Worktree
- Task/Job
- Pipeline, Stage, Gate, Approval
- Tool, Skill
- Artifact, Capsule
- Policy, Capability Lease
- Event, Audit Entry
- Execution Backend
- Secret / Env Var

See: `03_kernel_contract.md`, `05_orchestration_ontology.md`, `agent/06_agent_system.md`.

## 7) Modes vs capabilities

ModuPrompt is a multi-mode workspace, but **modes are ultimately capabilities on top of the kernel**.

### MVP mode: Orchestrator (Command Center)

- Board(s) that visualize projects, sessions, tasks, gates, approvals.
- Worktree-centric workflows.
- Deterministic pipeline execution.
- CLI-first automation and headless operation.

### Later capabilities (plugin-like)

- **Context engineering / prompt graph** (node-based prompt composition)
- **MCP management** (inspect/debug tool servers)
- **Skills hub** (discover/install/sync skills across agent CLIs)
- **Web Librarian** (high-performance web-to-document engine)

These are all expected to integrate via tools/skills and the event log.

## 8) MVP scope (first deliverable)

MVP ships a working kernel + CLI + minimal desktop UI shell focused on orchestration:

- Daemon skeleton
- Command API (typed SDKs; generic wire)
- Event store + projections
- Workspace + project management
- Session spawn/fork + lineage
- Worktree operations (create/list/delete/status/diff/commit)
- Task lifecycle + retries + timeouts
- Capsules generation and sharing rules
- Tool registry (search/load/run) + strict schema validation
- Hooks pipeline (WASM sandbox) at least for policy/audit and tool gating

## 9) Enterprise roadmap (high-level)

- Server mode (Postgres baseline) with multi-user auth (OIDC/SAML), ABAC enforcement
- Immutable audit stream with signing checkpoints
- Execution isolation (UID separation + container/pod execution)
- Web UI for collaboration
- Cluster profile (NATS JetStream)

## 10) Non-goals (explicit)

- Not trying to be an IDE.
- Not a Git hosting product.
- Not a generic workflow engine for all domains.

---

## References

[agentskills-spec]: https://agentskills.io/specification "Agent Skills Specification"
