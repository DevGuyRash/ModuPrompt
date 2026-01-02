# Orchestration Ontology (Boards, Pipelines, Stages, Gates, Tasks)

**Status:** Canonical.

This document defines the core orchestration concepts used by the MVP.

## 1) Definitions

### Workspace
A container for projects, policies, users (enterprise), shared skills, and storage.

### Project
A logical unit of work, often mapping to one or more git repositories.

### Board
A board is both:
1) a **visual canvas** (nodes + edges + rich cards), and
2) a **staging surface** for pipeline state (stages/gates/approvals displayed and manipulated on the board).

### Pipeline
A pipeline is a staged lifecycle model applied to tasks/sessions.

Pipelines exist at three levels:
- Global templates (reusable)
- Project-bound pipelines (configured defaults)
- Run instances (immutable per workflow execution)

### Stage
In v1, each task has a single current stage label. The data model supports upgrading to a DAG of stage nodes later.

### Gate
A gate is a blocking condition that must be satisfied to advance stages.

Types include:
- manual approval
- policy decision
- tool predicate satisfied
- webhook wait
- delay/cron

### Approval
An approval is a signed/attributed event that unblocks a gate.

Approvers may be:
- human
- deterministic policy-bot
- AI scorer (nondeterministic, but audited and captured)

### Task
A task is a tracked unit of work.

Tasks may be:
- atomic (single agent run)
- composite (workflow step that spawns multiple agent runs)

## 2) Minimum stage types (v1)

- plan
- implement
- test
- review
- merge
- security_gate
- manual_approval
- external_webhook_wait
- delay_or_cron

## 3) Advancing work: the rule

**Only the kernel may advance state.**

Agents may propose stage transitions, but the daemon applies them only after:
- policy evaluation
- required approvals
- gate predicates satisfied

## 4) Retries / timeouts / deadlines

### Retries
Retries are first-class and include:
- attempt count
- max attempts
- backoff strategy
- jitter
- failure classification

### Timeouts
Timeouts exist at:
- tool call level
- task level
- stage level

### Deadlines / SLAs
Tasks and/or stages may include deadlines:
- deadlines are required fields even if enforcement is partial in v1

## 5) Commands and events (outline)

This ontology is enforced via commands and events.

Examples (illustrative, not exhaustive):

Commands:
- `task.create`
- `task.advance_stage`
- `gate.request_approval`
- `approval.grant`
- `task.retry`

Events:
- `task.created`
- `task.stage_changed`
- `gate.blocked`
- `approval.granted`
- `task.failed`

See deep-dives:
- `orchestration/boards.md`
- `orchestration/pipelines.md`
- `orchestration/tasks.md`
