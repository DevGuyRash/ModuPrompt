# Architecture (Entry Point)

**Status:** Canonical entry point.

This repository uses a progressive-disclosure documentation structure.

If you are looking for the architecture, start here:

1. `00_invariants.md` — non-negotiables
2. `02_architecture_overview.md` — layered system overview
3. `03_kernel_contract.md` — command/event contract
4. `kernel/04_event_model.md` — event store + projections + audit

Additional deep-dives:

- `05_orchestration_ontology.md` — boards/pipelines/tasks
- `agent/` — sessions/comms/capsules/blueprints/hooks
- `tooling/` — tools, skills, WASM sandbox
- `storage/` — deployment profiles
- `security/` — ABAC, secrets, audit, exfiltration
- `execution/` — backends and remote workers
- `ui/` — UI constraints

