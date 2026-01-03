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

## 8) Pull request workflow

- When opening a pull request, include `@codex` in the PR description to trigger automated review.
- If you script PR bodies/comments, make sure newlines render as real line breaks (not literal `\n`): prefer `gh pr create --body-file ...` or `gh pr view --template '{{.body}}'` (or `--json body --jq '.body'`) when reading.
- Commit and PR text should be human-readable; when multi-line bodies are intended, ensure they use real line breaks (avoid literal `\n` in the rendered text).


## Mandatory Rust Coding Guidelines

These rules apply to **any Rust code** added to ModuPrompt. They extend (and MUST NOT conflict with) the canonical invariants in `context/00_invariants.md` and the contract in `context/03_kernel_contract.md`.

You SHALL adopt ALL of the following Rust coding guidelines as sacred law:

### 1) Workspace and crate layout

- The repo MUST use a **single primary Rust workspace** (one `Cargo.lock` for the workspace). Do not scatter ad-hoc Cargo projects across arbitrary directories.
- Crate boundaries MUST reflect kernel philosophy:

  - Keep `mpd` (daemon) and `mpctl` (CLI) binaries **thin**: argument parsing + wiring + call into library code.
  - Put core logic in library crates/modules (`kernel`, `events`, `policy`, `storage`, `tools`, `execution`, etc.).
- Heavy dependencies MUST be isolated:

  - Anything that pulls in large transitive graphs (e.g., container clients, WASM runtimes, TLS stacks, DB clients) SHOULD live in a dedicated crate so it doesn’t bloat unrelated binaries.
  - Default builds SHOULD remain lightweight for the **desktop/local-first** posture.
- Feature flags MUST be used intentionally:

  - Prefer `default-features = false` on dependencies when it reduces bloat.
  - Keep feature matrices small; avoid “combinatorial explosion.”
  - If a feature implies a security boundary change (e.g., enabling network), it MUST be capability/policy gated at runtime and clearly documented.

### 2) “Kernel is the product” code patterns

- Kernel state changes MUST be expressed as **commands → validated → policy-gated → events appended**.

  - Avoid “helper” code paths that mutate state without going through the same command/event machinery.
  - If it didn’t go through the daemon, it didn’t happen.
- Prefer a functional core for state transitions:

  - Command handling SHOULD be structured so that the core logic is close to:

    - `fn handle(cmd, state) -> Result<Vec<Event>>`
  - Side effects (tool execution, filesystem writes, network access) MUST happen **outside** the pure transition logic and MUST be recorded as events/artifacts.
- Projections MUST be rebuildable:

  - Projection code MUST be idempotent and deterministic.
  - Projections MUST NOT become a second source of truth.

### 3) Determinism and replay discipline

Determinism is not optional: **the same event log MUST reproduce the same resulting state**.

- Do not allow nondeterminism to leak into state:

  - Any nondeterministic output (LLM responses, external tool stdout, timestamps, randomness, environment-dependent behavior) MUST be captured as **artifacts and/or events** so replay is deterministic.
- Use stable ordering everywhere it matters:

  - Avoid `HashMap`/`HashSet` iteration when output is user-visible, hashed, serialized, or persisted in a way that affects replay. Prefer `BTreeMap`/`BTreeSet` or explicitly sort.
  - For lists returned to clients, prefer stable sorting (and document the sort key).
- Time and randomness MUST be handled explicitly:

  - “Now” SHOULD be injected (a `Clock` interface) and recorded in events where relevant; don’t call `SystemTime::now()` deep in logic that affects state.
  - Randomness used for security (e.g., secret keys) MUST be cryptographically strong; randomness used for behavior MUST be captured so replay matches.
- Serialization MUST be stable where it impacts hashing or reproducibility:

  - If you hash serialized data, you MUST ensure canonical bytes (stable key ordering, stable formatting rules) or hash the canonical source bytes directly.
  - Prefer explicit `\n` line endings for generated textual artifacts to avoid cross-platform diffs.

### 4) Security-by-default implementation rules

- Fail closed by default:

  - Unknown input fields MUST be rejected.
  - Policy evaluation failures MUST default to deny (unless explicitly configured for local debugging, and that configuration MUST be hard to accidentally ship).
- No secrets in logs/projections:

  - Code MUST treat any secret material as toxic to logs, projections, and UI previews.
  - Redaction MUST happen at boundaries (command ingestion, tool results, audit logging), not as an afterthought.
- Deny-by-default network posture:

  - Network access MUST be treated as a capability and MUST be mediated by kernel tools/policy/audit (no “quick HTTP call” inside business logic).
  - If a crate introduces network-capable dependencies, the code MUST make it hard to use them outside the approved tool boundary.
- Filesystem access MUST respect scoping:

  - Kernel-mediated filesystem operations MUST enforce allowlists/denylists (worktree scoping, no reaching into `~/.ssh`, etc.).
  - Prefer capability-scoped filesystem APIs; avoid ad-hoc `std::fs` usage in high-level code paths unless it is clearly within an enforced scope.
- WASM sandbox boundaries MUST remain tight:

  - Hostcalls MUST be minimal, capability-scoped, and emit events (inputs/outputs redacted).
  - Do not add “convenient” hostcalls that become a backdoor for network or unrestricted filesystem access.

### 5) Schema discipline for commands, events, tools

Schemas are part of the product contract.

- Rust types that represent externally supplied payloads (commands, tool inputs, config) MUST reject unknown fields:

  - Use `#[serde(deny_unknown_fields)]` (or an equivalent enforcement) by default.
  - Avoid `#[serde(flatten)]` unless you can still enforce strictness.
- Events MUST follow the canonical envelope shape and versioning rules:

  - Schema changes SHOULD be additive when possible.
  - If you change an event payload, you MUST provide an upcaster strategy (or a clear version migration plan) consistent with deterministic replay.
- Errors MUST be structured and safe:

  - Machine-readable error codes SHOULD exist for command rejections and tool failures.
  - Error messages MUST avoid including secret material or sensitive file contents.

### 6) Error handling, logging, and observability

- Panics are not control flow:

  - Non-test Rust code MUST NOT use `unwrap()`/`expect()` except for truly impossible states, and those MUST be justified with an invariant comment.
- Prefer structured logging:

  - Use structured fields (e.g., `trace_id`, `workspace_id`, `session_id`, `tool_id`, `event_id`) so audit/debug is possible without scraping strings.
  - Logging MUST respect redaction rules.
- Correlation is mandatory:

  - Command handling SHOULD propagate `trace_id` through event emission and tool execution paths.
  - Tool execution MUST emit intent/gated/executed events (or their equivalents) with stable linkage.

### 7) Testing expectations (deterministic, hermetic)

- Tests SHOULD be deterministic and hermetic:

  - Use temp dirs and fixtures.
  - Avoid network in tests; if unavoidable for a specific tool, isolate it behind mocks/recordings and keep it out of default test runs.
- Minimum coverage expectations for new features:

  - At least one happy path test.
  - At least one meaningful rejection/error path test (e.g., policy deny, unknown field rejection, schema mismatch).
  - For any public JSON surface, include a characterization test that proves unknown fields are rejected.
- Prefer testing observable behavior:

  - For CLI: stdout/stderr, exit codes, JSON output shape.
  - For daemon/kernel: emitted events, projection updates, and idempotency behavior.

### 8) Performance and scalability (without cleverness)

- Optimize for predictable performance, not microbench heroics:

  - Avoid O(n²) patterns in projections and board snapshot generation.
  - Stream large logs/artifacts; don’t slurp entire outputs into memory unless bounded and justified.
- Concurrency MUST not break ordering guarantees:

  - You MAY use concurrency for execution/tool running, but event append ordering MUST remain correct and reproducible (`seq_global` semantics).
  - Any concurrent processing that affects emitted events MUST be carefully structured so output ordering is deterministic.

### 9) Unsafe code policy

- Unsafe SHOULD be avoided by default.
- If unsafe is required (FFI, low-level perf, runtime integration), it MUST be:

  - isolated to a small module/crate,
  - documented with explicit safety invariants,
  - covered by tests that exercise the unsafe boundary,
  - reviewed with a security mindset (assume hostile inputs).

### 10) Cross-platform and “no runtime sprawl” constraints

- Desktop/local-first MUST not require Node/Python/JVM runtimes.
- OS-specific behavior MUST be gated and tested:

  - Use `cfg` responsibly; avoid silently degrading security on a platform.
  - Path handling MUST not assume UTF-8 unless explicitly required by the interface, and MUST avoid platform-specific path traversal pitfalls.

---

If a proposed Rust change makes any of the above hard to uphold (especially determinism, strict schemas, policy boundaries, or secret redaction), the change MUST either be redesigned or explicitly escalated as a potential invariant violation per `AGENTS.md`’s golden rule.
