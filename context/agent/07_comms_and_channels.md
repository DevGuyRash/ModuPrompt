# Communication (Lineage Scopes, Groups, Leases, Message Types)

**Status:** Canonical.

## 1) Principle: communication is mediated

Agents do not DM each other directly. They publish communication intents to the kernel:

- `message.send(scope, type, payload)`

The kernel enforces:
- ABAC policy
- lineage rules
- channel retention rules
- redaction rules

## 2) Session lineage model

Each session tracks:

- `session_id`
- `parent_session_id` (optional)
- `root_session_id`
- `lineage_path` (e.g., `/root/child/grandchild`)

## 3) System scopes (always available)

- `PARENT`
- `CHILDREN`
- `SIBLINGS`
- `DESCENDANTS`
- `PROJECT`
- `WORKSPACE`

## 4) Default rules

- Child → parent: allowed (within capsule types)
- Parent → children: allowed
- Sibling → sibling: allowed for capsules by default
- Cousins: **not allowed by default**

## 5) Escape hatches

### 5.1 Group channels (persistent)

Groups are first-class objects:

- explicit membership
- ABAC-enforced
- retention controls

Groups are the primary mechanism for cross-tree collaboration.

### 5.2 Capability leases (temporary)

Leases are TTL capability tokens that grant scoped exceptions:

- subject(s) allowed
- message types allowed
- TTL / expiration
- audit trail and reason

Only trusted hooks/policies may mint leases; sandboxed hooks may request lease issuance.

## 6) Message types

Message types are strict enums with namespaced extensions.

Core types include:

- `status_capsule`
- `plan_capsule`
- `decision_capsule`
- `artifact_manifest`
- `risk_flag`
- `human_comment`
- `tool_intent`
- `tool_result`
- `policy_notice`

Extensions:

- `x.<org>.<name>` (e.g., `x.company.custom_event`)

## 7) Retention

Minimal v1 retention controls:

- per-session TTL
- per-channel TTL
- per-group TTL

Enterprise may add legal hold and retention exports.
