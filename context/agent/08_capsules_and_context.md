# Capsules & Context (Progressive Disclosure, Hard Budgets)

**Status:** Canonical.

## 1) Goal

Make context reusable and scalable without blowing token budgets.

**Rule:** Default sharing uses capsules and manifests, not raw transcripts.

## 2) Capsule types (required)

Each session automatically emits:

- **Status capsule** (tiny, frequent)
- **Plan capsule** (on plan change)
- **Decision capsule** (key decisions)
- **Artifact manifest** (files/patches/reports/log refs)
- **Risk capsule / flag** (policy/security concerns)

These are stored as artifacts and referenced in events.

## 3) Hard budgets enforced by the kernel

Budgets are applied per source, at the kernel boundary, e.g.:

- status capsule: <= 1,500 chars
- plan capsule: <= 3,000 chars
- decision capsule: <= 3,000 chars
- artifact manifest: <= 2,000 chars (expandable via references)

Budgets are configuration, but the enforcement mechanism is mandatory.

## 4) Transcript access

- Full transcripts are not shared by default.
- Transcript access requires explicit capability + redaction policies.
- The kernel may provide “transcript slices” under strict budgets.

## 5) Progressive disclosure pattern

All high-cardinality resources use the same pattern:

1) `*.search(query)` → IDs + short summaries
2) `*.load(id)` → full definition/content (budgeted)
3) `*.run(id,args)` → execution

This applies to:

- tools
- skills
- capsules
- artifact indexes

## 6) Programmatic tool calling (token-efficient)

Complex multi-tool logic should run outside the LLM context window when possible:

- Hooks/WASM modules may request tool calls programmatically.
- Intermediate data stays in artifacts/capsules instead of being pasted into prompt context.

---

## References

[anthropic-advanced-tool-use]: https://www.anthropic.com/engineering/advanced-tool-use "Advanced tool use"
[anthropic-code-exec-mcp]: https://www.anthropic.com/engineering/code-execution-with-mcp "Code execution with MCP"
