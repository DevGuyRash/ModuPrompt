# PRD - Orchestrator Mode (Sessions, Worktrees, Boards, Pipelines)

**Status:** Draft.

## 1) Purpose

Ship the MVP mode: a command center for multi-agent workflows with deterministic pipelines, worktree-centric isolation, and lineage-scoped communication.

## 2) Goals

- Visual orchestration universe: multiple projects, session trees, live status.
- Worktree-per-session when useful, with one-writer locks and many readers.
- Deterministic pipeline with stages, gates, and approvals.
- Spawn and fork semantics that preserve lineage and reproduce state.
- Headless-first: everything available via API + CLI; GUI is a client.
- External CLI agents supported in baseline mode (PTY capture, isolation, timeouts).

## 3) Non-goals (v1)

- Fully featured collaborative editing UX
- In-app merge conflict resolution UI
- Advanced pipeline DAG editor (beyond simple stage labels)

## 4) Personas

- Solo power user (local desktop)
- Senior engineer/team lead orchestrating parallel worktrees
- Platform/DevOps user integrating automation via CLI/webhooks

## 5) User stories

- As a user, I can create projects and attach repos.
- As a user, I can spawn sessions (agents) with a blueprint and watch them run.
- As a user, I can fork a session into an alternate branch (new worktree/new branch by default).
- As a user, I can stage work through plan/implement/test/review/merge with explicit gates.
- As a user, I can approve or deny transitions and see provenance.
- As a user, I can see capsule summaries from siblings without reading full transcripts.

## 6) Functional requirements

### Boards
- Boards represent both a canvas and a staging surface.
- Boards support 10k+ node overviews with virtualization and LOD.

### Sessions
- Sessions form a lineage tree (root/parent/child).
- Sessions emit capsules automatically.

### Worktrees
- Many sessions can attach to a worktree; only one writer at a time.
- Worktree operations (create/list/delete/status/diff/commit) exist in v1.

### Pipelines, stages, gates
- Minimum stage set: plan/implement/test/review/merge plus: security gate, manual approval, webhook wait, delay/cron.
- Advancing stages is kernel-mediated; agents may only propose.
- Approvals may be issued by humans, policy-bot, and AI scorer (audited).

### Tasks
- Tasks have retries, timeouts, and deadlines fields.
- Tasks may be composite (spawn sub-tasks and sessions).

## 7) Commands and events (minimum)

Commands:
- `board.create`, `board.update_layout`
- `session.spawn`, `session.fork`, `session.attach_worktree`, `session.detach_worktree`
- `worktree.create`, `worktree.list`, `worktree.delete`, `worktree.status`, `worktree.diff`, `worktree.commit`
- `task.create`, `task.advance_stage`, `task.retry`, `task.cancel`, `task.pause`, `task.resume`
- `gate.request_approval`
- `approval.grant`, `approval.revoke`

Events:
- `board.created`, `board.layout_updated`
- `session.spawned`, `session.forked`, `capsule.updated`
- `worktree.created`, `worktree.writer_locked`, `worktree.writer_released`
- `task.created`, `task.stage_changed`, `task.blocked`, `task.completed`, `task.failed`
- `gate.blocked`, `approval.granted`, `approval.revoked`

## 8) Non-functional requirements

- Live updates via event stream without UI frame drops.
- Clear provenance: for every transition, show policy decision refs and artifacts.
- Deterministic replay: orchestration state rebuilds from event log.

## 9) Milestones

1. Board projection + session tree projection.
2. Spawn session + attach worktree (one writer lock enforced).
3. Task create + stage changes + gate blocks.
4. Approvals and policy gating integrated.
5. CLI can watch and manage all orchestration operations.

## 10) Risks

- Cross-platform PTY consistency.
- Worktree correctness across edge cases.
- Performance on 10k node boards.

