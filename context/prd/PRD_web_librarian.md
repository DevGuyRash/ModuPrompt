# PRD - Web Librarian (Content Aggregator)

**Status:** Draft (post-MVP capability).

## 1) Purpose

Provide a high-performance, concurrent, local-first web-to-document engine that integrates with the kernel as a capability (tools + artifacts + events).

## 2) Goals

- Rust-first crawler/parser with strong concurrency.
- Reliable extraction via configurable rules and optional AI-assisted parsing.
- Persistent library with versioned snapshots and diffs.
- Outputs: Markdown + HTML bundle first; EPUB/PDF later (optional).

## 3) Non-goals (v1 librarian)

- Full JS-rendered site support (heavy mode later)
- Aggressive anti-bot circumvention

## 4) Functional requirements

### Library
- Track sources/series with metadata.
- Detect changes and new chapters.
- Snapshot versions with timestamps.
- Diff against prior snapshots.

### Parsing
- Manual extraction: selectors, regex patterns, content block picking.
- AI-assisted parsing (remote API first; local later).

### Outputs
- Export to Markdown and HTML bundle (v1).
- Later exports: EPUB/PDF/DOCX.

### Integration with kernel
- Crawler runs as tasks.
- Results stored as artifacts (content-addressed) + indexed.
- Events emitted for updates, failures, and exports.

## 5) Non-functional requirements

- Concurrency: 1,000+ concurrent fetches (configurable).
- No UI stalls: run off the UI thread.
- Respect rate limits and retries.

## 6) Milestones

1. URL ingest + extraction rules.
2. Snapshot store + diff.
3. Export MD/HTML.
4. Optional AI-assisted parsing.

---

## References

[webtoepub]: https://github.com/dteviot/WebToEpub "WebToEpub"
