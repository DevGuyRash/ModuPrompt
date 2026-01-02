# Webhooks & Triggers (Incoming + Outgoing)

**Status:** Canonical.

Webhooks are a first-class automation surface. Most meaningful state transitions must be webhook-able.

## 1) Outgoing webhooks (events -> external)

### 1.1 Model

- Outgoing webhooks are subscriptions to event types (and optional filters).
- Delivery semantics are **at-least-once**.
- Receivers must treat events as idempotent by `event_id`/sequence.

### 1.2 Signing (required)

- Webhook payloads must be signed (HMAC) with a per-webhook secret.
- Include timestamps and replay protection.

### 1.3 Delivery reliability

- Store delivery attempts and results.
- Retry with backoff and max attempts.
- Maintain a dead-letter queue (DLQ) for failures.
- Support manual replay.

## 2) Incoming webhooks (external -> actions)

Incoming webhooks can trigger deterministic actions under policy.

### 2.1 Allowed actions

Incoming webhooks may (policy controlled):

- trigger workflows
- enqueue tasks
- propose stage transitions
- request tool executions

### 2.2 Auth and verification

- All incoming webhooks must be authenticated (token + signature).
- Rate limit and log all attempts.

## 3) Event compatibility

Webhook payloads mirror the canonical event envelope (or a stable subset).

## 4) Audit and privacy

- Webhook deliveries are audited.
- Secrets are never included in webhook payloads.

