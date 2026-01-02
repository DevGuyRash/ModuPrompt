# Hooks (Interceptor Pipeline) — Phases, Trust, Failure Policy

**Status:** Canonical.

Hooks are the core of the “agent builder.” They are first-class middleware that can:

- rewrite/redact messages
- block/allow actions
- clamp scopes and capabilities
- request approvals
- summarize results into capsules

Hooks exist both for human-authored policy logic and for user-defined automation (sandboxed).

## 1) Hook phases (all required in v1)

1. **Pre-Plan** — before agent begins (inject constraints, enforce stage gates)
2. **Pre-Send** — before sending to the model/CLI (rewrite, redact)
3. **Post-Receive** — after agent output (validate, score, reject, capsule generation)
4. **Pre-Tool** — before tool execution (gating, approvals, argument transforms)
5. **Post-Tool** — after tool result (summarize, create artifacts)
6. **State Transition** — stage/gate changes (gates, approvals)
7. **Spawn/Fork** — enforce who can create descendants
8. **Audit** — always-on capture and redaction

## 2) Trust tiers

1. **Trusted hooks** (admin-installed, signed)
   - may mint capability leases
   - may execute broader hostcalls based on policy

2. **Sandboxed hooks** (default)
   - run in WASM sandbox with explicit capabilities
   - cannot mint leases; can only request lease issuance

3. **Proposed hooks** (agent-generated)
   - require approval before activation (unless single-user mode opts into auto-accept)

## 3) Hook capabilities (v1)

Hooks MAY:

- rewrite content (message/tool arguments)
- block/allow
- clamp scopes (e.g., reduce FS allowlists)
- request approvals
- request tool calls via `request_tool_call` hostcall (kernel executes + gates)

Only **trusted hooks** MAY:

- mint capability leases

## 4) Failure policy

- Default: **fail closed** (especially enterprise)
- Local dev may opt into fail-open for debugging only

Failing closed means:
- the action is rejected
- a hook failure event is emitted

## 5) Test harness (required in v1)

Hook testing must support:

- replay recorded inputs through a hook chain
- compare outputs (including budget enforcement)
- fuzz budgets and malformed inputs
- produce deterministic reports and artifacts

## 6) Observability

Every hook invocation must be logged as events:

- inputs (redacted)
- decisions (allow/block/transform)
- outputs (redacted)
- timing

---

## References

[wasmtime-security]: https://docs.wasmtime.dev/security.html "Wasmtime Security"
