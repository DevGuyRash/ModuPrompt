# Secrets (Encryption at Rest, Scopes, Injection, Redaction)

**Status:** Canonical.

## 1) Goals

- Secrets are always encrypted at rest.
- Secrets never appear in plaintext logs, projections, or UI unless explicitly revealed.
- Default injection avoids env vars.
- Policy controls every access.

## 2) Scopes and precedence

Secrets exist at:

- global
- user
- project
- session

Default lookup precedence:

`session > project > user > global`

Tools SHOULD prefer explicit scope targeting to avoid ambiguity.

## 3) Encryption design (envelope encryption)

### 3.1 Data keys per secret

- Each secret is encrypted with a randomly generated data key (DEK).
- The DEK is wrapped by a master key (KEK).

### 3.2 Master key (KEK) sources

Desktop profile:
- optional per-workspace vault password derives KEK
- unlock is per-workspace; auto-lock on idle; re-auth for sensitive actions

Enterprise profile:
- a server-managed KEK, designed to be KMS-backed later

### 3.3 Password-derived KEK (desktop vault)

If using a vault password:

- derive the KEK using a memory-hard KDF (Argon2id)
- store salts and parameters with the vault metadata
- support parameter upgrades over time

## 4) Storage strategy: encrypted raw + redacted derivative

For searching and UI previews:

- store encrypted raw secret value
- store a redacted derivative (e.g., `****` + metadata) for indexing

Never store plaintext values in projections.

## 5) Injection mechanisms (preferred order)

1. **stdin** (preferred)
2. **file** (temporary file with tight permissions, if required)
3. **env vars** (discouraged; compatibility-only)

All injection requires:

- ABAC authorization
- audit events
- redaction policy

## 6) Rotation and versioning

- Secrets are versioned (rollback possible).
- Rotation emits events and triggers dependent task/session updates where permitted.

---

## References

[rfc-9106]: https://datatracker.ietf.org/doc/rfc9106/ "RFC 9106: Argon2 Memory-Hard Function"
