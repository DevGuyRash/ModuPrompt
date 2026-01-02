# Skills (Packaging, Registry, Signing)

**Status:** Canonical.

Skills are portable capability bundles that package deterministic capabilities in a shareable format.

A skill may contain:

- documentation (progressive disclosure)
- tool schemas and examples
- scripts and/or WASM modules
- tests
- references

ModuPrompt provides a **skill registry** to search, load, and execute skills.

## 1) Skill directory convention (v1)

A skill is a directory with at minimum:

- `SKILL.md` — YAML frontmatter + markdown body

Optional:

- `wasm/` — WASM modules (hooks/tools)
- `scripts/` — deterministic scripts (only via kernel runner)
- `refs/` — documentation references
- `tests/` — harness inputs/expected outputs

## 2) Registry operations (progressive disclosure)

- `skill.search(query)` → skill IDs + short summaries
- `skill.load(skill_id)` → definition + exposed tools + examples (budgeted)
- `skill.run(skill_id, input)` → deterministic execution

## 3) Trust and signing

Skills have trust tiers:

- **trusted**: signed by an admin/trusted key; may be granted broader capabilities
- **sandboxed**: default; runs with minimal capabilities
- **proposed**: created/generated and awaiting approval

Signing is optional in v1, but **required for the trusted tier**.

## 4) Execution rules

- Skills do not mutate state directly.
- All state mutation occurs through kernel tools or kernel-mediated APIs.
- Script/WASM execution is governed by the same policy and audit model as tool calls.

## 5) Sync across agent CLIs (capability)

The kernel provides deterministic operations to sync skills across external agent CLIs:

- install to global or workspace scope
- pin versions per workspace/project
- deploy via symlinks where supported; copy otherwise
- generate tool-specific adapters/shims when needed
- detect drift via hashes and emit events

This is implemented as tools/skills (never as hidden side effects).

---

## References

[agentskills-spec]: https://agentskills.io/specification "Agent Skills Specification"
