## Summary

- 

## Changes

- 

## Validation

- [ ] `cargo fmt --all -- --check` (if Rust workspace present)
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings` (if Rust workspace present)
- [ ] `cargo test --workspace --all-features` (if Rust workspace present)
- [ ] `python scripts/check_docs.py`

## Invariants checklist

- [ ] Reviewed `context/00_invariants.md` and did not violate invariants (or documented changes).
- [ ] Reviewed `context/03_kernel_contract.md` for command/event compatibility.
- [ ] Reviewed `context/security/19_security_architecture.md` for security implications.
- [ ] Docs updated and reference labels included per `context/references.md`.
