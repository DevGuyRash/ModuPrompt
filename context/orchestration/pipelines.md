# Pipelines (Templates → Project Binding → Run Instances)

**Status:** Canonical.

## 1) Pipeline lifecycle

Pipelines exist at three levels:

1) **Global pipeline templates**
   - reusable across projects/workspaces
   - versioned

2) **Project-bound pipelines**
   - choose a template + project-specific overrides
   - defines defaults (e.g., required approvals)

3) **Run instances**
   - created per workflow execution
   - immutable reference to:
     - pipeline template version
     - policy bundle version
     - tool registry version
     - hook chain versions

## 2) Stage model (v1 → future)

### v1

- A task has a single `current_stage` label from the stage set.
- Stages may have associated gate definitions.

### Future-ready

- The schema supports upgrading to a pipeline DAG where:
  - stage nodes have entry/exit predicates
  - edges represent permitted transitions

## 3) Gate types

- Manual approval gate
- Policy evaluation gate
- Tool predicate gate
- Webhook wait gate
- Delay/cron gate

## 4) Approvals model

Approvals may be issued by:

- Humans
- Deterministic policy-bot
- AI scorer (nondeterministic, audited)

Approvals are events with:

- approver identity
- scope (what gate/stage/task)
- reason
- optional evidence artifact references

## 5) Transition semantics

- Stage transitions are proposed via command.
- Kernel evaluates:
  - ABAC policy
  - gate satisfaction
  - required approvals
- Kernel either:
  - emits a stage-change event, or
  - emits a rejection + gate-blocked event
