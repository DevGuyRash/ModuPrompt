# Canonical Invariants (Non‑Negotiables)

**Status:** Canonical.

This document lists requirements that must remain true across implementation and future features.

## 1) The Kernel is the Product

1. A single-writer **daemon** is the system of record.
2. GUI/web/TUI/CLI are **clients**.
3. **If it didn’t go through the daemon, it didn’t happen.**

## 2) Event-Sourced Truth + Deterministic Replay

1. All meaningful state transitions are immutable **events**.
2. Projections (indexes, views, search) are derived.
3. **Determinism baseline:** the same event log must reproduce the same resulting state.
4. Nondeterministic outputs are captured as artifacts/events so replay becomes deterministic.

## 3) Propose → Gate → Execute

1. Agents propose structured actions.
2. The kernel gates actions via policy (ABAC), approvals, and hooks.
3. The kernel executes deterministic actions (tools/scripts) and records results.

## 4) Strict Tool Contracts

1. Kernel tools are invoked only via **schema-valid inputs**.
2. Unknown fields are rejected (fail closed).
3. Every tool declares a side-effect taxonomy (pure/workspace/filesystem/network/process/git/container).

## 5) Progressive Disclosure + Hard Budgets

1. No “context dumping.”
2. Tools/skills/context follow a universal pattern: **search → load → run**.
3. The kernel enforces hard budgets per context source (capsules, manifests, transcripts).

## 6) Lineage‑Scoped Communication

1. Agents do not DM arbitrarily.
2. Communication is mediated by the kernel and enforced by policy.
3. Default: **no cousin communication**.
4. Escape hatches:
   - persistent cross-tree collaboration via **group channels**
   - temporary scoped exceptions via **capability leases (TTL)**

## 7) Capsules Over Transcripts

1. Default shared artifacts are structured capsules:
   - status, plan, decision, artifact manifest, risk flags
2. Full transcripts are not shared by default and are always permissioned + redacted.

## 8) Security by Default

1. **ABAC from day one**; RBAC roles compile into ABAC rules.
2. Network access is denied by default; allowlists + approvals for network tools.
3. Secrets are encrypted at rest; logs and indexes store redacted derivatives.
4. Immutable audit stream exists separately from the core event stream.
5. Tamper evidence uses hash-chaining and periodic signed checkpoints.

## 9) Extensibility Without Runtime Sprawl

1. Desktop profile targets near-zero runtime dependencies.
2. Optional integrations (git, docker/podman, agent CLIs) are capability-gated.
3. User-defined code runs in a sandboxed **WASM runtime** by default.

## 10) Identifiers and Artifacts

1. Entities/events use time-sortable IDs (preferred: **UUIDv7**).
2. Artifacts are content-addressed via **BLAKE3** hashes.

---

## References

[rfc-9562]: https://www.rfc-editor.org/rfc/rfc9562.html "RFC 9562: Universally Unique IDentifiers (UUIDs)"
[blake3]: https://github.com/BLAKE3-team/BLAKE3 "BLAKE3 official repo"
