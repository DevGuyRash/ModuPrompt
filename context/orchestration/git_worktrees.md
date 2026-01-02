# Git + Worktrees (Hybrid Strategy)

**Status:** Canonical.

## 1) Goals

- Worktrees are central to isolation and reproducibility.
- Correctness beats ideology: exact behavior should match real git behavior.
- Desktop profile keeps "zero runtime dependencies" by making git optional unless worktrees are used.

## 2) Strategy

### 2.1 Hybrid implementation

- Prefer Rust-native implementations for read-only operations where practical:
  - status
  - diff
  - commit metadata queries

- Allow calling system `git` for edge cases and for exact compatibility, especially:
  - worktree create/delete/repair
  - tricky ref/worktree edge cases

### 2.2 Capability gating

- Git operations are tools with taxonomy `git`.
- A blueprint/session must be granted `git` tool permissions.
- If `git` is unavailable on the host, worktree features are disabled or require explicit setup.

## 3) Worktree writer lock

- Only one session may have write permissions to a worktree at a time.
- Lock acquisition/release is kernel-mediated and emits events.

## 4) Reproducibility

- Record:
  - repo URL/identity
  - worktree path and branch
  - commands invoked
  - tool versions where feasible

This supports deterministic replay of orchestration state.
