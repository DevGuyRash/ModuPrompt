# ModuPrompt Style & Code Review Guidelines

**Status:** Canonical.

> Canonical review + style guidance for **humans and AI reviewers**.
> For deeper detail, also follow `AGENTS.md` and the contract/spec docs under `context/`.
> See `AGENTS.md` § 8 for PR workflow (including required AI reviewers).
> See `AGENTS.md` § 7 for TDD requirements, § 8 for design principles (SOLID/DRY/KISS/YAGNI), § 9 for complexity analysis.

## Quick rules (priority order)

1) **Do not break canonical invariants** (`context/00_invariants.md`).
   - **Invariant #1: Single-writer daemon** owns state. Clients (CLI/UI) must not write the DB directly.
   - **Invariant #2: Event log is truth**. Projections are derived and rebuildable.
   - **Invariant #3: Propose → Gate → Execute**. Agents propose; the kernel gates; only the kernel executes and records.
   - **Invariant #8: Deny-by-default security** (ABAC, no surprise network, no secrets in logs).

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
   - No "hidden" state. If replay needs it, capture it as an event field or artifact reference.
   - Avoid `HashMap`/`HashSet` iteration where ordering affects persisted/output data; sort or use `BTree*`.
   - Concurrency MUST NOT break event ordering guarantees (`seq_global` semantics).

5) **Security & secrets**
   - Never log secret material; treat it as toxic.
   - Network is a capability: no ad‑hoc HTTP calls in business logic; route via kernel tools with policy + audit.

6) **Reviewer output expectations**
   - Label findings: **BLOCKER / MAJOR / MINOR / NIT**.
   - For BLOCKER/MAJOR, suggest a concrete fix (pseudo‑diff or exact API change) and name the invariant it protects.
   - Prefer small, composable patches; avoid broad rewrites.

### Severity definitions

| Label | Meaning |
|-------|---------|
| **BLOCKER** | Violates a canonical invariant or introduces a security vulnerability. Must fix before merge. |
| **MAJOR** | Breaks contract (schema, API), creates nondeterminism, or degrades security posture. Must fix before merge. |
| **MINOR** | Style issues, test coverage gaps, or documentation omissions. Should fix, but non-blocking. |
| **NIT** | Cosmetic or optional improvements. Author discretion. |

---

## Review checklist by change type

### A) Command / event changes (contract surface)

- [ ] Payload schema added/updated in `schemas/**` (versioned, additive if possible).
- [ ] Unknown fields rejected (schema + `deny_unknown_fields`).
- [ ] Characterization test proving unknown fields are rejected.
- [ ] Idempotency enforced for state-changing commands.
- [ ] `expected_version` / conflict behavior tested if applicable.
- [ ] `trace_id` propagation verified through event emission and tool execution.
- [ ] Event(s) emitted are sufficient to rebuild projections deterministically.
- [ ] Projections updated to handle new events, and rebuild remains idempotent.
- [ ] Docs updated: `context/03_kernel_contract.md` and any referenced PRDs.

### B) Storage / projections

- [ ] Projections are derived only from events; no extra sources of truth.
- [ ] Rebuild works from a cold DB (drop projections → replay events).
- [ ] Queries and outputs use stable ordering (no `HashMap`/`HashSet` iteration affecting output).
- [ ] Migrations are safe and tested (SQLite WAL where relevant).

### C) Tools / hooks / policy boundaries

- [ ] Side-effecting behavior is a kernel tool (schema'd, gated, audited).
- [ ] Hooks fail closed; trust tiers respected (sandboxed vs trusted).
- [ ] WASM sandbox hostcalls are minimal, capability-scoped, and emit events.
- [ ] No new network capability without explicit allowlists + approvals.

### D) CLI / UI

- [ ] Binaries stay thin (arg parsing + wiring).
- [ ] JSON output is stable; exit codes are meaningful.
- [ ] No direct DB writes; all via daemon API.

### E) Network / external calls

- [ ] No new network capability without explicit allowlists + policy gating.
- [ ] HTTP calls route via kernel tools (no ad-hoc HTTP in business logic).
- [ ] Network-capable deps isolated behind feature flags.

### F) Docs & repo hygiene

- [ ] `python scripts/check_docs.py` passes (references and links).
- [ ] `cargo fmt`, `cargo clippy -D warnings`, `cargo test`, `cargo deny` are clean.
- [ ] `cargo bench` for perf-sensitive changes (if applicable).

---

## Rust style & correctness expectations (summary)

- No `unwrap()`/`expect()` in non-test code unless *provably impossible* (add invariant comment).
- Prefer explicit error types; keep errors structured and non-leaky (no secrets, no full file contents).
- Add tests for: happy path + rejection path (schema/policy/conflict).
- Add characterization tests proving unknown fields are rejected for any public JSON surface.
- Keep dependency bloat isolated behind crates/features; default desktop build stays lightweight.
- Concurrency MUST NOT break event ordering guarantees (`seq_global` semantics).

---

## Test quality (reviewers verify)

> For TDD workflow (Red-Green-Refactor), see `AGENTS.md` § 7.

- [ ] Tests exist for new features and bug fixes (TDD: test written before fix).
- [ ] Tests are **deterministic** and **hermetic** (no flaky tests).
- [ ] Tests are **non-brittle**: test behavior, not implementation details.
  - Avoid over-mocking; prefer real collaborators where practical.
  - Don't assert on incidental details (exact error messages, internal field names).
  - Tests should survive refactoring if behavior is unchanged.
- [ ] Tests are **readable**: clear arrange-act-assert structure, descriptive names.
- [ ] Tests are **fast**: no network, no sleeps, no unnecessary I/O.
- [ ] Edge cases and error paths are covered, not just happy paths.

---

## Design principles & complexity (reviewers verify)

> For full guidance, see `AGENTS.md` § 8 (SOLID/DRY/KISS/YAGNI) and § 9 (complexity).

- [ ] Code follows **SOLID**, **DRY**, **KISS**, **YAGNI** principles.
- [ ] Time and space complexity documented for non-trivial algorithms.
- [ ] O(n²) or worse algorithms justified with bounded-input rationale.
- [ ] No unbounded memory growth; streaming preferred for large data.
- [ ] Iterators preferred over intermediate `Vec` allocations where practical.

---

## Key spec references

For detailed requirements, see:

- `AGENTS.md` — TDD workflow (§ 7), design principles (§ 8), complexity analysis (§ 9)
- `context/00_invariants.md` — canonical non-negotiables
- `context/03_kernel_contract.md` — command/event envelope structure
- `context/kernel/04_event_model.md` — event store, projections, replay
- `context/security/19_security_architecture.md` — ABAC, policy, audit
- `context/tooling/12_tools_and_schemas.md` — tool contracts, side-effect taxonomy
