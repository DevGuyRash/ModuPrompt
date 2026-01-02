# Glossary

**Status:** Canonical.

- **Daemon / Kernel**: The single-writer process (`mpd`) that owns state changes, enforces policy, and emits events.
- **Command**: A request submitted by a client/agent to the daemon; validated and either applied or rejected.
- **Event**: An immutable record of a meaningful state transition; the source of truth.
- **Projection**: A derived read model built from events (indexes, snapshots, search).
- **Artifact**: Stored output (logs, diffs, reports, capsules) referenced by content hash.
- **Capsule**: A structured summary artifact (status/plan/decision/manifest) designed to be shareable under hard budgets.
- **Session**: The unit of agent conversation + execution; sits in a lineage tree.
- **Worktree**: A git worktree used as an execution scope; may be attached to many sessions but only one writer.
- **Task**: A tracked unit of work, atomic or composite, with retries/timeouts/deadlines.
- **Pipeline**: A staged lifecycle template applied to tasks/sessions.
- **Stage**: The current pipeline label for a task in v1; DAG stage nodes later.
- **Gate**: A blocking condition required before stage advancement.
- **Approval**: An attributed event that unblocks a gate.
- **Hook**: A middleware interceptor phase that can rewrite/block/clamp actions; runs in WASM by default.
- **Tool**: A deterministic kernel operation described by strict schema; executed by kernel.
- **Skill**: A packaged capability bundle (docs + schemas + scripts/WASM) in a portable format.
- **ABAC**: Attribute-based access control; the primary authorization model.
- **RBAC**: Role-based access control; compiled into ABAC bundles.
- **Lease**: A TTL capability token granting temporary scoped permission.
