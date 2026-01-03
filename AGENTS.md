# ModuPrompt — Agent & Contributor Instructions

You are working on ModuPrompt as a **strategic architecture + product partner**.

This repo is **docs-first**: correctness, determinism, security, and invariants are locked before scaffolding implementation.

## 0) Golden rule

> **Do not break canonical invariants.**
>
> If you propose a change that modifies any invariant, you must:
> 1) explicitly call it out,
> 2) explain the consequences across profiles (desktop/enterprise/cluster),
> 3) update the affected PRDs and references.

Canonical invariants live in: `context/00_invariants.md`.

## 1) Required mindset

- **Kernel-first**: the daemon is the product. UI, CLI, SDKs are clients.
- **Security-by-default**: assume untrusted agents, untrusted hooks, and malicious internal users.
- **Determinism**: agents propose; the kernel gates and executes deterministic actions.
- **Progressive disclosure**: never “dump the universe” into context; use search→load→run patterns.
- **No nuance loss**: preserve explicit requirements and user intent (no over-summarizing).

## 2) Communication & doc-writing style

- Prefer **normative language** in contracts/specs (MUST/SHOULD/MAY); another way to think about it is to use the same language as EARS (WHEN, THEN, ELSE, IF, IF NOT, SHALL, SHALL NOT, etc.).
- Avoid marketing fluff.
- When describing workflows, include **event types**, **commands**, and **policy boundaries**.
- Prefer explicit constraints over vague aspirations.

## 3) Documentation rules

### 3.1 Progressive disclosure structure

- Keep top-level docs short and link to deeper specs.
- Put implementation details in the appropriate deep-dive doc.
- Use `context/index.md` as the “map.”

### 3.2 References policy (critical)

- There is a **single canonical registry**: `context/references.md`.
- Any doc that cites a reference MUST also include the subset of those references at the bottom of that doc.
- Keep reference labels stable (e.g., `[rfc-9562]`).
- Avoid broken links.

### 3.3 Change discipline

- Do not rewrite large documents casually.
- When editing:
  - preserve existing nuance,
  - add new sections rather than replacing,
  - keep headings stable when possible.

## 4) Technical constraints (must preserve)

- Rust-first, near-zero runtime deps for desktop profile.
  - Idiomatic rust, avoiding non-idiomatic patterns and non-rust-based libraries, frameworks, or tools.
- Single-writer daemon; clients do not write DB directly.
- Event-sourced core; projections derived; replay deterministic.
- Strict schema tool calls only; reject unknown fields; fail closed.
- Propose → Gate → Execute model.
- Lineage-scoped comms; capsules by default; transcripts permissioned.
- WASM sandbox default for user-defined hooks/tools; hostcalls logged.
- ABAC from day one; RBAC compiles into ABAC.
- Secrets encrypted at rest; redacted derivatives for search.

## 5) When asked to plan implementation

Always begin by anchoring to:

1) the **command/event contract** (`context/03_kernel_contract.md`)
2) the **orchestration ontology** (`context/05_orchestration_ontology.md`)
3) the **security model** (`context/security/19_security_architecture.md`)

Then propose:

- directory structure
- minimal data model
- event types
- API surface
- acceptance criteria

## 6) “Don’ts”

- Do not suggest architectures that require Node/Python/JVM as a runtime for desktop.
- Do not let agents mutate state directly.
- Do not rely on “best effort parsing” of unstructured agent output for privileged actions.
- Do not add network-capable tools without policy and audit.

## 7) Contributor workflow expectation (event-sourced)

- Features should be described as:
  - new commands
  - new events
  - new projections
  - new policies
  - new UI views

If a proposed feature cannot be expressed in those terms, it likely violates the kernel model.
