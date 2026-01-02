# Tools & Schemas (Strict Tool Contract)

**Status:** Canonical.

Tools are deterministic operations executed by the kernel.

Agents and hooks may **request** tool calls, but only the kernel executes them.

## 1) Tool definition

Each tool is described by:

- `id` (namespaced)
- `title`, `description`
- `input_schema` (JSON Schema)
- `output_schema` (JSON Schema)
- `examples` (good calls + typical errors)
- `safety_notes` (human-readable constraints, redaction concerns)
- `side_effect_taxonomy` (required)

## 2) Strict validation (fail closed)

- Tool inputs MUST validate against schema.
- Unknown fields are rejected.
- Tool errors are structured, machine-readable, and include safe metadata.

## 3) Side-effect taxonomy

Every tool declares one of:

- `pure`
- `workspace`
- `filesystem`
- `network`
- `process`
- `git`
- `container`

Tool permissions can be granted/denied by taxonomy and by tool ID.

## 4) Progressive disclosure primitives

All clients and agent runtimes use:

1) `tool.search(query)` → IDs + short summaries
2) `tool.load(id)` → schema + examples + safety notes
3) `tool.run(id,args)` → executes (kernel only)

## 5) Permission model

Permissions are the intersection of:

- global policy
- project policy
- blueprint baseline
- session restrictions

Explicit deny wins.

## 6) Deterministic execution envelope

Every tool call produces:

- `tool.intent` event (requested)
- `tool.gated` event (allowed/denied + reasons)
- `tool.executed` event (result summary + artifact refs)

Tool results that are large must be stored as artifacts; events carry references.

---

## References

[json-schema-spec]: https://json-schema.org/specification "JSON Schema Specification"
