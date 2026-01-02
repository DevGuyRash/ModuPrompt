# WASM Sandbox Model (Hooks + Tools)

**Status:** Canonical.

WASM is the default programmable substrate for:

- hooks (middleware/interceptors)
- user-defined tools
- policy modules (later)

The goal is **programmability without handing raw system access**.

## 1) Runtime

- Use an embedded WASM runtime (e.g., Wasmtime) compiled into the daemon binary.
- Modules are executed with a **capability-based hostcall API**.

## 2) Capabilities

Each module executes with an explicit capability set:

- filesystem scope (allow/deny paths; read/write)
- tool request permissions (side-effect taxonomy)
- capsule read/write permissions and budgets
- timeouts and CPU/memory limits
- random/crypto APIs
- (optional) approvals request ability

## 3) Minimal hostcalls (v1)

Allowed hostcalls:

- `emit_event(type, payload)`
- `read_capsule(id)`
- `write_capsule(type, content)`
- `request_tool_call(tool_id, args)`
- `filesystem_api(op, path, ...)` (scoped)
- `crypto_random(len)`
- `clock_now()` / `sleep(ms)` (deterministic options later)
- `request_approval(kind, reason, evidence_refs...)` (optional)

Not allowed:

- raw network sockets

**Network access must occur via kernel tools** so policy + audit always apply.

## 4) Deterministic logging

Every hostcall must automatically emit an event record including:

- module id + version hash
- hostcall name
- inputs (redacted)
- outputs (redacted)
- timing

This ensures:

- replayability
- auditability
- provenance tracking

## 5) Budget enforcement

Capsule and context budgets must be enforced at hostcall boundaries.

If a module exceeds budget:

- the hostcall fails
- the hook/tool fails closed (by default)
- a budget violation event is emitted

## 6) Trust tiers and signing

- Trusted modules may have broader capabilities.
- Sandboxed modules get minimal capability sets.
- Proposed modules require approval.

Signed module support is required for the trusted tier.

---

## References

[wasmtime-security]: https://docs.wasmtime.dev/security.html "Wasmtime Security"
