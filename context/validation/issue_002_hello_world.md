# Validation — Issue 002 (Initial Implementation)

**Status:** Checklist.

This document captures the minimal validation path for Issue #2 (daemon skeleton + event store + command API + projections + CLI connect).

## 1) Build

- `cargo build --workspace`

Expected:
- `mpd` and `mpctl` binaries are produced by Cargo.

## 2) Hello-world path (manual)

1. Start the daemon:
   - `mpd start`
2. Confirm daemon is reachable:
   - `mpctl daemon status`
3. Create a workspace:
   - `mpctl workspace init ./demo --name demo`
4. List workspaces:
   - `mpctl workspace list --json`
5. Watch event stream (cursor catch-up + tail):
   - `mpctl events watch --workspace demo --from 0`
6. Create a project:
   - `mpctl project create --workspace demo --name core`
7. List projects:
   - `mpctl project list --workspace demo --json`
8. Restart the daemon and verify projections rebuild:
   - Stop `mpd`, then `mpd start`
   - `mpctl workspace list --json` should still return the workspace.

Expected:
- Events stream shows `workspace.created` and `project.created`.
- Projections survive restart (rebuild from event log).

## 3) Test suite (minimum)

- `cargo test -p mp-daemon`

Expected:
- `workspace_persists_across_restart`
- `idempotency_reuses_events`
- `expected_version_mismatch_rejects`
- `event_stream_catches_up`

## 4) Acceptance criteria coverage

- `mpd` and `mpctl` binaries exist and run.
- Command schema validation rejects unknown fields.
- Idempotency keys prevent duplicate state changes.
- `expected_version` conflicts yield explicit rejection.
- Realtime stream supports cursor catch-up by `seq_global`.
- Projections rebuild from event log on restart.
