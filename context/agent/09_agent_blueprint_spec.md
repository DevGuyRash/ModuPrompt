# AgentBlueprint Specification (Agent Builder)

**Status:** Canonical.

An `AgentBlueprint` is the **first-class definition of an agent**.

It defines:

1. Model/runtime adapter (external CLI, API model, internal runtime)
2. Toolset (built-in kernel tools + skills)
3. Context policy (what can be loaded, budgets)
4. Communication policy (scopes/groups/leasing)
5. Hook chain (interceptors/middleware)
6. Resource limits (timeouts, concurrency, token budgets)
7. Execution sandbox (filesystem allowlists, network policy, backend)
8. Version pinning for reproducibility

## 1) Storage and authoring

- Source of truth is a blueprint file in the workspace (git-friendly).
- The daemon maintains an index of blueprints in the DB for fast lookup.

## 2) Version pinning (non-negotiable)

Every session records and pins:

- blueprint ID + version
- hook chain IDs + versions/hashes
- tool registry version
- skill bundle versions/hashes
- policy bundle hash/version
- pipeline template version (if applicable)

## 3) Suggested file format (v1)

Blueprint files are YAML.

### 3.1 Top-level structure

```yaml
api_version: 1
id: reviewer
name: "Reviewer"
description: "Reviews patches and produces decision capsules."

# Version pinning
pins:
  policy_bundle: "pol_01..."
  tool_registry: 1
  skills:
    - id: "org.core.git"
      version: "sha256:..."  # or content hash

runtime:
  kind: external_cli   # external_cli | internal | remote
  adapter: "claude-code"  # adapter id
  config:
    command: ["claude", "--json"]
    env_injection: "stdin" # stdin | env | file

execution:
  backend: local_process   # local_process | docker | podman | k8s | remote_worker
  workdir_scope:
    mode: worktree
    allow_write: true
  filesystem:
    allow:
      - "worktrees/**"
    deny:
      - "**/.ssh/**"
  network:
    default: deny
    allow_domains:
      - "api.github.com"

context:
  budgets:
    status_capsule_chars: 1500
    plan_capsule_chars: 3000
    decision_capsule_chars: 3000
    manifest_chars: 2000
  sources:
    allow:
      - status_capsules
      - plan_capsules
      - decision_capsules
      - artifact_manifests
    transcript:
      allow: false

comms:
  scopes:
    allow:
      - PARENT
      - CHILDREN
      - SIBLINGS
      - PROJECT
  groups:
    allow_join: ["reviewers"]
  leases:
    request_allowed: true

tools:
  allow_taxa:
    - workspace
    - git
    - filesystem
  deny_tools:
    - "network.http"  # example

hooks:
  chain:
    - id: "core.audit"
      version: "1.0.0"
      trust: trusted
    - id: "core.redaction"
      version: "1.0.0"
      trust: sandboxed
    - id: "core.tool_gate"
      version: "1.0.0"
      trust: trusted

limits:
  max_concurrency: 2
  tool_timeout_ms: 60000
  task_timeout_ms: 900000
```

### 3.2 Notes

- `pins.*` are required in enterprise profiles; local dev may allow floating pins only when explicitly configured.
- `tools.allow_taxa` restricts what classes of tools can be invoked.
- `execution.*` defines the sandbox boundary.

## 4) Blueprint lifecycle

- Blueprints are loaded from workspace files.
- Changes produce events and create new versions.
- Sessions reference the exact blueprint version used.

## 5) GUI editing

The GUI may edit blueprint files, but the daemon remains the writer and emits events that reflect blueprint changes.
