# Policy Engine & DSL (ABAC Core)

**Status:** Canonical.

## 1) Goals

- ABAC enforcement from day one.
- Human-manageable roles (RBAC) compile into ABAC rule bundles.
- Policy must be evaluated at every security boundary.
- Policies should be portable across desktop/enterprise/cluster profiles.

## 2) Policy evaluation points

The daemon MUST evaluate policy at:

- command submission
- spawn/fork
- message send (scope/group)
- tool gating (pre-tool)
- filesystem access
- secrets access/injection
- network tool use
- stage transitions / approvals

## 3) Core concepts

### Subjects
- users (enterprise)
- sessions (agents)
- system actors (policy-bots)

### Objects
- workspaces/projects
- sessions/tasks/worktrees
- artifacts/capsules
- secrets
- tools
- channels/groups

### Actions
- create/read/update/delete
- spawn/fork
- send_message
- request_tool
- execute_tool
- read_secret
- inject_secret
- network_access
- approve_gate

## 4) DSL direction (v1)

v1 uses a Rust-first declarative DSL with:

- explicit allow/deny rules
- match predicates on subject/object/action
- budget constraints (e.g., transcript slices)
- capability constraints (e.g., only allow git tools)

Example (illustrative):

```text
deny  subject.session.role != "orchestrator"  action == "worktree.create"
allow subject.session.project == object.project  action in {"tool.request","tool.execute"}  where tool.taxon in {"git","filesystem"}
allow subject.user.in_group("reviewers") action == "approve_gate" where gate.type == "review"
```

## 5) Advanced modules (later)

- WASM policy modules may be introduced for complex logic, but must still be deterministic and auditable.

## 6) Output

Policy evaluation produces:

- allow/deny decision
- structured rationale
- optional required approvals

Decisions are referenced in events and written to the audit stream.
