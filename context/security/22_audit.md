# Audit Stream (Immutable, Exportable, Tamper-Evident)

**Status:** Canonical.

## 1) Why separate audit

A separate audit stream provides defense-in-depth:

- hardened retention policies
- export tooling
- clear separation from application projections

Audit entries reference core event IDs.

## 2) What is audited (minimum)

- all state-changing commands
- all policy decisions (allow/deny + rationale)
- all tool gating decisions
- all tool executions (summary + artifact refs)
- all secret access/injection events (redacted)
- all network tool use (destination + allowlist decision)
- all approvals and overrides
- all capability leases minted and used
- all admin policy changes

## 3) Tamper evidence

- Core event log supports hash chaining.
- Audit stream supports periodic signed checkpoints over event ranges.

## 4) Retention and export

- Per-workspace retention policies.
- Enterprise: legal holds and exports to immutable storage targets.

## 5) Relationship to projections

Audit is not a projection and must not be rebuildable from projections.
It may be rebuildable from the core event log only if the deployment explicitly allows it.

