# PRD - Security and Enterprise Readiness

**Status:** Draft.

## 1) Purpose

Ensure the system is enterprise-compatible from day one: ABAC enforcement, secrets handling, auditability, and safe execution boundaries.

## 2) Goals

- ABAC from day one; RBAC roles compile into ABAC.
- Policy evaluated at all boundaries (commands, tools, FS, network, comms, spawn/fork, stage transitions, secrets).
- Deny-by-default network; allowlists at global/project/blueprint/session; explicit deny wins.
- Secrets encrypted at rest; redacted derivatives for search; injection via stdin by default.
- Separate immutable audit stream with signed checkpoints.
- Execution isolation options: per-user Unix separation and/or container/pod backends.

## 3) Non-goals (v1)

- Full compliance certifications (SOC2/FedRAMP/etc.)
- Advanced DLP classifiers (planned later)

## 4) Functional requirements

### Authentication (enterprise)
- OIDC/SAML login.
- API tokens for automation.
- Bootstrap admin setup.

### Authorization
- ABAC policy engine with a Rust-first declarative DSL.
- Capability leases (TTL bearer tokens) for temporary scoped exceptions.

### Secrets
- Envelope encryption (DEK per secret; KEK master key) with KMS-ready design.
- Per-workspace vault unlock for desktop profile.

### Audit
- Separate append-only audit stream that references core event IDs.
- Tamper evidence: hash chain + periodic signed checkpoints.

### Exfiltration controls (v1 minimum)
- Block secret material from entering prompts/messages by policy.
- Require approvals for any network-capable tool.
- Domain allowlists enforced and audited.

## 5) Non-functional requirements

- Fail closed defaults for policy and hooks.
- No plaintext secrets in logs/projections/UI.
- Deterministic replay of security-relevant state from event log.

## 6) Milestones

1. Policy engine evaluation points wired across commands/tools/comms.
2. Secrets encryption + injection mechanisms.
3. Audit stream + signed checkpoints.
4. Enterprise auth integration.

## 7) Risks

- Policy complexity creep; keep DSL small and deterministic.
- Cross-platform differences for isolation primitives.

---

## References

[rfc-9106]: https://datatracker.ietf.org/doc/rfc9106/ "RFC 9106: Argon2 Memory-Hard Function"
