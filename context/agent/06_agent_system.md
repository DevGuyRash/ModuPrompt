# Agent System (Sessions, Worktrees, Spawn/Fork)

**Status:** Canonical.

## 1) What an “agent” is

An agent is a runtime-adapted worker that can:

- propose actions (tools, stage transitions, spawns)
- produce artifacts and capsule updates
- communicate within policy-enforced scopes

MVP agents are primarily wrappers around external CLIs (observed and isolated). Over time, an internal agent runtime will provide deeper interception and richer tool calling.

## 2) Sessions

A **session** is the unit of conversation + execution. It contains:

- transcript events (permissioned)
- structured message events
- tool intents/results
- capsule updates
- lineage metadata (parent/child)
- links to tasks and worktrees

## 3) Worktree attachment model

Worktrees are not strictly 1:1 with sessions.

- Many sessions may attach to a single worktree.
- Only **one writer** is allowed at a time; others are read-only/advisory.
- Writer ownership is a kernel-enforced lock.

## 4) Spawn

Spawning creates a child session.

- A session may spawn only if it holds a capability that allows spawning.
- Spawn targets may be any project in the workspace if permitted by policy.
- Spawn emits events that link parent and child into a lineage tree.

## 5) Fork

Forking creates an alternate branch of work from an existing session state.

Supported fork modes:

- **Default:** new worktree + new branch + new session
- **Same-branch fork:** new worktree on the same branch + new session
- **Planning-only fork:** new session without a worktree

Fork preserves lineage: the new session records its fork parent, and the system maintains a session tree.

## 6) Interruption model

- Internal runtime agents (later) can support cooperative mid-stream interrupts.
- External CLIs (v1) support:
  - SIGINT attempt
  - then kill + restart with new constraints as the reliable fallback
