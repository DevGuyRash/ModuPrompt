# Local Profile (SQLite)

**Status:** Canonical.

## 1) Why SQLite locally

SQLite enables a true local-first desktop profile without requiring a running DB server process.

## 2) Concurrency strategy

- **Single writer**: the daemon owns all writes.
- Multiple readers: GUI/CLI connect via daemon and read projections.
- WAL mode enables readers during writes.

## 3) Layout

- events table (append-only)
- projections tables (derived)
- artifact index table (content-addressed)
- idempotency table (per command)
- (optional) FTS tables

## 4) Startup/migrations

- daemon runs migrations on start
- automatic safe backup before migration
- safe-mode recovery uses event replay

---

## References

[sqlite-when-to-use]: https://sqlite.org/whentouse.html "Appropriate Uses For SQLite"
[sqlite-wal]: https://sqlite.org/wal.html "Write-Ahead Logging"
