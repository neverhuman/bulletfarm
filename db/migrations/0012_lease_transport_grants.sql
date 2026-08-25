-- Durable signed lease-transport grant index. Readback after farmd restart
-- returns the last acquire for an idempotency digest. This is not a public
-- /v1 route and is not a launch grant.

CREATE TABLE lease_transport_grants (
  idempotency_digest TEXT PRIMARY KEY CHECK (
    typeof(idempotency_digest) = 'text'
    AND length(idempotency_digest) = 64
    AND idempotency_digest NOT GLOB '*[^0-9a-f]*'
  ),
  grant_json TEXT NOT NULL CHECK (typeof(grant_json) = 'text' AND length(grant_json) > 0),
  recorded_at TEXT NOT NULL
);
