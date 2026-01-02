# Boards (Canvas + Staging Surface)

**Status:** Canonical.

## 1) Board responsibilities

A board is the primary orchestration surface. It must:

- visualize projects/sessions/worktrees/tasks and their relationships
- visualize pipeline stage and gate state
- provide controls to propose state transitions (which the kernel gates)
- display realtime updates via the event stream

## 2) Board content model (v1)

### Node types

Boards contain typed nodes, for example:

- **Session node**: an agent session, lineage links, status capsule, runtime state
- **Task node**: a task with stage/gate summary and lifecycle status
- **Worktree node**: a worktree attach point (writer lock, readers, repo status)
- **Artifact node**: reports/patches/build logs; content-addressed
- **Gate node**: a blocking gate with approval state
- **Group/frame node**: container for organizing nodes
- **Comment node**: human commentary and approvals rationale

### Edge types

- lineage edges (parent → child sessions)
- task dependency edges (optional in v1)
- worktree attachment edges (session/task → worktree)
- artifact provenance edges (task/session → artifact)

## 3) Stage representation

A board must support stage visualization:

- lanes/columns (kanban-like) for v1 single-stage labels
- gate markers on nodes (blocked, pending approval, waiting webhook, etc.)
- future: DAG stage node view

Moving a node between lanes is a **proposal**; it results in a command such as `task.advance_stage`.

## 4) Realtime

Boards are live views:

- The board subscribes to the event stream.
- Projections produce efficient “board snapshots” for initial load.
- Clients must support catch-up from a `seq_global` cursor.

## 5) Scale targets

- Typical: 200–2,000 nodes per board.
- Heavy: 10,000+ nodes on overview boards.

The UI must use:

- virtualization (render only what’s visible)
- level-of-detail rendering when zoomed out
- throttled layout and incremental updates
