# Future Capability - Context Engineering / Prompt Graph Mode

**Status:** Captured vision (not MVP).

This document preserves the planned context engineering mode so the orchestrator MVP can later consume it as a capability.

## 1) Node-based workflows

- Users build prompts on a canvas (node graph), similar to node-based editors in creative tools.
- Prompts are modular with typed inputs/outputs that connect to form a final prompt.
- Many node types exist (variables, prompt blocks, filters, aggregators, switches, references, etc.).

## 2) Dedicated editors (important UX invariant)

- Editing long prompts inside a node is painful.
- The node editor must provide:
  - a popout modal for node text editing
  - a dedicated general-purpose editor tab reused across features

## 3) Diagramming objects (Mermaid + PlantUML)

- Diagrams are first-class objects with stable IDs.
- Prompts can insert diagram objects via placeholders.

Example placeholder:

- `[Chart <ID-truncated...>]`

When compiling a final prompt package:
- placeholders are resolved and replaced with full Mermaid/PlantUML content.

## 4) Prompt injection / template syntax (initial proposal)

The bracket type determines token semantics:

- `{{...}}` for Variable Nodes (primitive values)
- `[[...]]` for Prompt Nodes (inject another prompt block)
- `<<...>>` for reference nodes or semantic keywords (final mapping TBD)

Rules:

- syntax must be unambiguous and safe in raw text
- unresolved tokens produce validation warnings and link to the expected node
- editor auto-completes tokens based on bracket type

## 5) Semantic keyword dictionary (reusable phrases)

A dictionary of reusable prompt fragments:

- fields: `keyword`, `type`, `description`, `value`, optional `constraints`
- tagging and grouping by domain (tone, format, safety, audience)
- versioning so nodes can pin to a specific revision

## 6) Easy mode: prompt composer (vision)

- builds prompts by selecting semantic keywords/phrases
- enforces required keywords per template
- outputs a prompt node or composite prompt
- optionally explains "why" selections were made

## 7) Brainstorm mode (generative diagramming)

A generative layer in the diagramming interface:

- user remains fully manual by default
- AI is an on-demand accelerator invoked explicitly

Actions:

- Connect (Converge): link concepts, add bridging nodes if needed
- Expand (Diverge): generate child nodes
- Bulk synthesis: tidy a soup of nodes into a network

UX:

- retrospective animation replay of generation steps
- strict vs creative toggles (max bridge depth, branching factor)
- review mode for accept/reject/prune generated clusters

## 8) Integration with the kernel

This mode should integrate as capabilities:

- graphs stored as artifacts + projections
- compilation produces a structured prompt package with provenance
- tool and skill usage is mediated by the kernel
