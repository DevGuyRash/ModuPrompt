# ModuPrompt Style & Code Review Guidelines

> Canonical review + style guidance for **humans and AI reviewers**.
> For deeper detail, also follow `AGENTS.md` and the contract/spec docs under `context/`.

## Quick rules (priority order)

1) **Do not break canonical invariants** (`context/00_invariants.md`).
   - **Single-writer daemon** owns state. Clients (CLI/UI) must not write the DB directly.
   - **Event log is truth**. Projections are derived and rebuildable.
   - **Propose → Gate → Execute**. Agents propose; the kernel gates; only the kernel executes and records.
   - **Deny-by-default security** (ABAC, no surprise network, no secrets in logs).

2) **Anything that changes state MUST go through:**
   - command payload schema validation (reject unknown fields),
   - policy/hook gates (fail closed),
   - event emission (append-only),
   - projection updates derived from events.

3) **Schemas are part of the product contract.**
   - JSON Schemas MUST set `additionalProperties: false`.
   - Rust structs for external payloads MUST use `#[serde(deny_unknown_fields)]`.
   - Add characterization tests proving unknown fields are rejected.

4) **Determinism & replay discipline**
   - No “hidden” state. If replay needs it, capture it as an event field or artifact reference.
   - Avoid `HashMap`/`HashSet` iteration where ordering affects persisted/output data; sort or use `BTree*`.

5) **Security & secrets**
   - Never log secret material; treat it as toxic.
   - Network is a capability: no ad‑hoc HTTP calls in business logic; route via kernel tools with policy + audit.

6) **Reviewer output expectations**
   - Label findings: **BLOCKER / MAJOR / MINOR / NIT**.
   - For BLOCKER/MAJOR, suggest a concrete fix (pseudo‑diff or exact API change) and name the invariant it protects.
   - Prefer small, composable patches; avoid broad rewrites.

---

## Review checklist by change type

### A) Command / event changes (contract surface)

- [ ] Payload schema added/updated in `schemas/**` (versioned, additive if possible).
- [ ] Unknown fields rejected (schema + `deny_unknown_fields`).
- [ ] Idempotency enforced for state-changing commands.
- [ ] `expected_version` / conflict behavior tested if applicable.
- [ ] Event(s) emitted are sufficient to rebuild projections deterministically.
- [ ] Projections updated to handle new events, and rebuild remains idempotent.
- [ ] Docs updated: `context/03_kernel_contract.md` and any referenced PRDs.

### B) Storage / projections

- [ ] Projections are derived only from events; no extra sources of truth.
- [ ] Rebuild works from a cold DB (drop projections → replay events).
- [ ] Queries and outputs use stable ordering.
- [ ] Migrations are safe and tested (SQLite WAL where relevant).

### C) Tools / hooks / policy boundaries

- [ ] Side-effecting behavior is a kernel tool (schema’d, gated, audited).
- [ ] Hooks fail closed; trust tiers respected (sandboxed vs trusted).
- [ ] No new network capability without explicit allowlists + approvals.

### D) CLI / UI

- [ ] Binaries stay thin (arg parsing + wiring).
- [ ] JSON output is stable; exit codes are meaningful.
- [ ] No direct DB writes; all via daemon API.

### E) Docs & repo hygiene

- [ ] `python scripts/check_docs.py` passes (references and links).
- [ ] `cargo fmt`, `cargo clippy -D warnings`, `cargo test`, `cargo deny` are clean.

---

## Rust style & correctness expectations (summary)

- No `unwrap()`/`expect()` in non-test code unless *provably impossible* (add invariant comment).
- Prefer explicit error types; keep errors structured and non-leaky (no secrets, no full file contents).
- Add tests for: happy path + rejection path (schema/policy/conflict).
- Keep dependency bloat isolated behind crates/features; default desktop build stays lightweight.
