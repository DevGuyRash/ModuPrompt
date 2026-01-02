# PRD - Tools, Skills, and MCP Strategy

**Status:** Draft.

## 1) Purpose

Provide deterministic operations (tools) and portable capability packaging (skills), with an optional path to MCP integration later.

## 2) Goals

- Strict JSON Schema tool definitions; reject unknown fields.
- Universal progressive disclosure primitives: search/load/run for tools and skills.
- Side-effect taxonomy declared on every tool, used for permissions.
- Skill registry supports Agent Skills format with optional signing; trusted tier requires signatures.
- MCP is optional and not required for MVP; initial MCP work is inspect/connect, not install.

## 3) Non-goals (v1)

- Full plugin store UI
- Automatically installing Node/Python/Docker-based tools without explicit user choice

## 4) Functional requirements

### Tools
- Define tools with input/output schemas, examples, safety notes.
- `tool.search`, `tool.load`, `tool.run` available to clients and agent runtimes.
- Tool execution produces intent/gate/execute events + artifacts.

### Skills
- Skill directory format with `SKILL.md` + optional wasm/scripts/tests.
- `skill.search`, `skill.load`, `skill.run` primitives.
- Trust tiers: trusted/sandboxed/proposed.
- Signature verification for trusted tier.

### MCP (later)
- Inspect/connect to existing MCP servers.
- Map MCP tools to kernel tool definitions with policy gating and audit.
- Optional installation and registry later, as an integration (not a runtime requirement).

## 5) Non-functional requirements

- Determinism: tools are replayable via events + artifacts.
- Security: tools and skills are policy gated; secret injection controlled.
- Scalability: progressive disclosure prevents context overload.

## 6) Milestones

1. Tool definition format + registry.
2. Strict schema validation + taxonomy-based permissions.
3. Skill registry + Agent Skills parsing.
4. Skill signing support for trusted tier.
5. Optional MCP inspection mode.

---

## References

[json-schema-spec]: https://json-schema.org/specification "JSON Schema Specification"
[agentskills-spec]: https://agentskills.io/specification "Agent Skills Specification"
[metamcp]: https://github.com/metatool-ai/metamcp "MetaMCP"
