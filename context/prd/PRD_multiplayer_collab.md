# PRD - Multiplayer / Collaboration

**Status:** Draft (enterprise later).

## 1) Purpose

Enable multiple users to view and operate in the same workspace concurrently with real-time updates and policy controls.

## 2) Goals

- Server-authoritative state (event log + projections).
- Clients subscribe to realtime and catch up from cursors.
- Shared boards and session visibility first; concurrent edits later.
- Collaboration is gated by ABAC and audited.

## 3) Non-goals (v1 collab)

- Full offline-first CRDT collaboration (later, only if needed)

## 4) Functional requirements

### Phase 1: Shared visibility
- Multiple users can view boards, sessions, and capsules.
- Comments and approvals are first-class events.

### Phase 2: Controlled multi-user edits
- Concurrency control uses optimistic concurrency (expected_version).
- Conflicts return explicit rejections.
- Optional resource-level locks for high-contention objects.

### Phase 3: Rich realtime collaboration
- Presence (who is viewing what)
- Cursor/selection sharing
- Live edits where safe

## 5) Non-functional requirements

- Correctness over perfect realtime: event log is truth, realtime is best-effort.
- Low-latency updates.
- Strong audit and retention controls.

## 6) Milestones

1. Multi-user auth + ABAC enforced across APIs.
2. Shared visibility + comments/approvals.
3. Controlled edits with conflict handling.
4. Presence and live collaboration enhancements.

