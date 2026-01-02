# PRD - CLI (and later TUI)

**Status:** Draft.

## 1) Purpose

Provide a first-class CLI that can operate the entire system headlessly, suitable for CI/CD, scripting, and automation.

## 2) Goals

- CLI talks to daemon; auto-starts it locally if not running.
- All commands support `--json` and stable exit codes.
- Streaming: `--watch` and event subscriptions.
- Batch mode: submit a workflow file and exit.
- No interactive prompts in non-interactive mode.

## 3) Non-goals (v1)

- Full-screen TUI (later)

## 4) Functional requirements

- `mpctl daemon status|start|stop`
- `mpctl workspace init|open|list`
- `mpctl project create|list`
- `mpctl session spawn|fork|list`
- `mpctl worktree create|list|delete|status|diff|commit`
- `mpctl task create|list|advance|cancel|retry`
- `mpctl events watch --json`

## 5) Non-functional requirements

- Composable output (JSON)
- Predictable exit codes by error class
- Low overhead (should not require runtimes)

## 6) Milestones

1. Auto-start daemon + ping.
2. Workspace/project/session commands.
3. `events watch` streaming.
4. Batch workflow submit.
5. TUI exploration (optional).
