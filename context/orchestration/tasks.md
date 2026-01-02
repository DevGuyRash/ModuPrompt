# Tasks (Lifecycle, Retries, Timeouts, Deadlines)

**Status:** Canonical.

## 1) Task types

- **Atomic task**: a single agent run or single tool execution (with structured inputs/outputs)
- **Composite task**: a workflow step that spawns multiple sub-tasks and/or agent runs

## 2) Task lifecycle states

Tasks support at minimum:

- queued
- running
- succeeded
- failed
- cancelled
- paused
- awaiting_user_input
- blocked_by_policy
- blocked_by_approval

## 3) Retries

Retries are first-class:

- `attempt`: current attempt number
- `max_attempts`
- `backoff`: fixed / exponential
- `jitter`: none / full / decorrelated
- `retry_on`: failure classifications (timeout, transient network, etc.)

Retry decisions may be automated by policy or require approval.

## 4) Timeouts

Timeouts exist at:

- tool call level
- task level
- stage level

Timeout expiration emits an event and transitions the task into a failure/block state depending on policy.

## 5) Deadlines / SLAs

Tasks may carry:

- `deadline_at`
- `priority`
- `sla_class`

Even if enforcement is partial in v1, these fields exist for enterprise readiness.

## 6) Evidence and artifacts

Tasks produce artifacts:

- logs (PTY capture)
- patches/diffs
- reports
- capsule updates

Artifacts are content-addressed and referenced by hash.
