# Security Architecture (ABAC, Capabilities, Exfiltration, Isolation)

**Status:** Canonical.

Security is not an add-on. The kernel is designed for enterprise deployment from day one.

## 1) Threat model (assume all)

- malicious internal users
- compromised client machines
- compromised agents / toolchains
- “honest-but-curious” operators

## 2) Auth (profiles)

### Desktop
- no login by default
- optional per-workspace vault unlock + re-auth on sensitive actions

### Enterprise
- OIDC/SAML (primary)
- API tokens for automation
- bootstrap local creds allowed only for initial setup

## 3) Authorization: ABAC core, RBAC compiled

- ABAC is the primary enforcement model.
- RBAC roles compile into ABAC rule bundles for operator-friendliness.
- Explicit denies win.

## 4) Capability tokens + leases

- Capabilities are recorded in the event log and enforced by the daemon.
- Temporary scoped exceptions use **bearer leases** with TTL.
- Leases are minted only by trusted hooks/policy (sandboxed hooks may request).

## 5) Policy evaluation points (must exist)

Policy must be evaluated at:

- command submission
- spawn/fork
- message send (scopes/groups)
- tool gating (pre-tool)
- filesystem access (kernel-mediated)
- secret access/injection
- network tool usage
- stage transitions and approvals

See: `21_policy_dsl.md`.

## 6) Network deny-by-default

- Network is a capability.
- Domain allowlists exist at global/project/blueprint/session levels.
- Effective policy is the **intersection**; explicit deny wins.
- No raw network hostcalls in WASM; network access happens only via kernel tools.

## 7) Secrets and redaction

- Secrets are encrypted at rest.
- Storage strategy is: **encrypted raw + redacted derivative** for search/index.
- Secret injection prefers stdin; env vars and files are compatibility-only and policy-controlled.

See: `20_secrets.md`.

## 8) Exfiltration controls (v1 minimum)

Must-have v1:

- deny-by-default network
- approvals for network tools
- secret redaction everywhere
- policy to block secrets entering prompts/messages

Enterprise later:

- deep content scanning/classifiers
- DLP-style rules and sensitive data detectors

## 9) Audit and tamper evidence

- Separate immutable audit stream (append-only)
- Hash-chained events + periodic signed checkpoints

See: `22_audit.md`.

## 10) Execution isolation

Defense-in-depth:

- per-user Unix identity separation where applicable (enterprise/Linux)
- optional rootless containers or k8s pods for high-risk workloads
- strict resource limits
- filesystem allowlists

See: `execution/23_backends.md`.
