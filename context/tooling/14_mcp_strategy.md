# MCP Strategy (Optional Capability)

**Status:** Canonical direction; not MVP-critical.

## 1) MVP stance

- MCP is not required for the core orchestrator MVP.
- The orchestrator MVP focuses on kernel primitives, tool registry, skills, hooks, and execution backends.

## 2) Phase 2 goals

When MCP support is introduced:

1) **Connect/inspect existing servers**
   - discover configured servers
   - start/stop/restart where safe
   - inspect request/response logs

2) **Deterministic tool mapping**
   - MCP tools become kernel tool definitions (schemas + safety notes)
   - MCP invocation remains behind policy and audit

3) **Optional install/manage**
   - allow optional add-ons that require Node/Python/Docker
   - never required for desktop runtime

## 3) Security posture

- MCP servers are treated as potentially unsafe external dependencies.
- Calls are mediated by kernel policy.
- Secrets are injected via stdin or tightly controlled mechanisms.

---

## References

[metamcp]: https://github.com/metatool-ai/metamcp "MetaMCP"
