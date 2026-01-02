# PRD - Agent Builder (Blueprints, Hooks, WASM)

**Status:** Draft.

## 1) Purpose

Deliver the "agent builder" differentiator: agents are assembled as pipelines of interceptors/middleware with policy-enforced execution, not as a single prompt blob.

## 2) Goals

- AgentBlueprint is the first-class agent definition with strict version pinning.
- Hook chain supports all phases (Pre-Plan, Pre-Send, Post-Receive, Pre-Tool, Post-Tool, State Transition, Spawn/Fork, Audit).
- Default programmable substrate for hooks and user tools is WASM, capability-scoped and logged.
- Hooks can request tool calls programmatically via `request_tool_call` hostcall (kernel executes + gates).
- Hook test harness exists in v1.

## 3) Non-goals (v1)

- Full marketplace/distribution ecosystem
- Advanced live debugging UI for hooks (beyond logs + harness)

## 4) Personas

- Power user authoring automation locally
- Enterprise admin authoring signed trusted hooks
- Teams building reusable agent templates

## 5) Functional requirements

### AgentBlueprint
- File is source of truth (workspace file), daemon maintains DB index.
- Blueprint pins: policy bundle, tool registry, hooks, skills, pipeline template.
- Blueprint defines: runtime adapter, execution backend, FS and network policy, context budgets, comms scopes/groups, tool taxons, limits.

### Hooks
- Hook phases are invoked at the right kernel boundaries.
- Hook outputs are structured (allow/block/transform + rationale).
- Fail closed by default; fail-open only via local debug configuration.

### Trust tiers
- trusted hooks: signed, may mint capability leases
- sandboxed hooks: default, cannot mint leases, can request
- proposed hooks: require approval

### WASM runtime
- Embedded runtime, no external runtime dependency.
- Minimal hostcalls: emit_event, read_capsule, write_capsule, request_tool_call, filesystem_api (scoped), crypto_random, clock/timers, request_approval.
- No raw network hostcalls.
- Every hostcall emits an event record.

### Hook test harness
- Replay recorded inputs through hook chains.
- Validate budget enforcement and schema compliance.
- Produce deterministic reports.

## 6) Non-functional requirements

- Security: sandboxed hooks cannot escape capabilities.
- Determinism: given the same artifacts and events, the same state is reproduced.
- Performance: hook execution must not block UI; hook timeouts enforced.

## 7) Milestones

1. Blueprint file schema + loader + indexing.
2. Hook pipeline with phase dispatch + fail-closed.
3. WASM runtime integration + hostcall layer + deterministic logging.
4. Tool gating hook + redaction hook + audit hook.
5. Hook test harness CLI.

## 8) Risks

- Sandbox escape or overly broad hostcalls.
- Complexity creep (keep v1 hostcalls minimal).
- Version pinning discipline.

---

## References

[wasmtime-security]: https://docs.wasmtime.dev/security.html "Wasmtime Security"
