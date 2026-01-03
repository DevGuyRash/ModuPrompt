# Reference Registry (Canonical)

This is the **single canonical** registry of external references.

**Rule:** Any doc that cites a reference MUST also include that reference label at the bottom of that doc.

## Standards / Specs

- [rfc-9562] RFC 9562 — Universally Unique IDentifiers (UUIDs) (incl. UUIDv7)
- [rfc-9106] RFC 9106 — Argon2 memory-hard function (password hashing / KDF guidance)
- [rfc-8895] RFC 8895 — Server-Sent Events (SSE)
- [ndjson-spec] NDJSON — Newline Delimited JSON specification
- [json-schema-spec] JSON Schema — Specification (2020-12)

## Storage / Messaging

- [sqlite-when-to-use] SQLite — Appropriate Uses
- [sqlite-wal] SQLite — Write-Ahead Logging
- [postgres-listen] PostgreSQL — LISTEN/NOTIFY
- [nats-jetstream] NATS — JetStream Concepts
- [redis-pubsub] Redis — Pub/Sub (notes on delivery semantics)

## Security / Runtime

- [wasmtime-security] Wasmtime — Security

## Tooling / Agents

- [claude-tool-use] Claude Docs — Tool use overview
- [claude-code-exec] Claude Docs — Code execution tool
- [anthropic-advanced-tool-use] Anthropic — Advanced tool use (engineering)
- [anthropic-code-exec-mcp] Anthropic — Code execution with MCP
- [agentskills-spec] Agent Skills — Specification

## Hashing / Crypto primitives

- [blake3] BLAKE3 — official repo
- [blake3-specs] BLAKE3 — specification repo

## Inspiration / Adjacent (non-binding)

- [metamcp] MetaMCP — MCP server manager/aggregator
- [webtoepub] WebToEpub — content aggregator reference

---

## Link definitions

[rfc-9562]: https://www.rfc-editor.org/rfc/rfc9562.html "RFC 9562: Universally Unique IDentifiers (UUIDs)"
[rfc-9106]: https://datatracker.ietf.org/doc/rfc9106/ "RFC 9106: Argon2 Memory-Hard Function for Password Hashing"
[rfc-8895]: https://www.rfc-editor.org/rfc/rfc8895.html "RFC 8895: Server-Sent Events"
[ndjson-spec]: https://github.com/ndjson/ndjson-spec "NDJSON Specification"
[json-schema-spec]: https://json-schema.org/specification "JSON Schema Specification"

[sqlite-when-to-use]: https://sqlite.org/whentouse.html "Appropriate Uses For SQLite"
[sqlite-wal]: https://sqlite.org/wal.html "Write-Ahead Logging"
[postgres-listen]: https://www.postgresql.org/docs/current/sql-listen.html "PostgreSQL LISTEN"
[nats-jetstream]: https://docs.nats.io/nats-concepts/jetstream "JetStream - NATS Docs"
[redis-pubsub]: https://redis.io/docs/latest/develop/pubsub/ "Redis Pub/Sub"

[wasmtime-security]: https://docs.wasmtime.dev/security.html "Wasmtime Security"

[claude-tool-use]: https://platform.claude.com/docs/en/agents-and-tools/tool-use/overview "Tool use with Claude"
[claude-code-exec]: https://platform.claude.com/docs/en/agents-and-tools/tool-use/code-execution-tool "Code execution tool - Claude Docs"
[anthropic-advanced-tool-use]: https://www.anthropic.com/engineering/advanced-tool-use "Introducing advanced tool use on the Claude Developer Platform"
[anthropic-code-exec-mcp]: https://www.anthropic.com/engineering/code-execution-with-mcp "Code execution with MCP"
[agentskills-spec]: https://agentskills.io/specification "Agent Skills Specification"

[blake3]: https://github.com/BLAKE3-team/BLAKE3 "BLAKE3 official repo"
[blake3-specs]: https://github.com/BLAKE3-team/BLAKE3-specs "BLAKE3 specification"

[metamcp]: https://github.com/metatool-ai/metamcp "MetaMCP"
[webtoepub]: https://github.com/dteviot/WebToEpub "WebToEpub"
