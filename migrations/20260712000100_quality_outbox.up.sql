-- Durable staging for quality's cross-module QualityInspectionCompleted disposition (outbox rollout plan,
-- P1). Stock SUBSCRIBES to this to accept/reject the inspected lot; a crash between the inspection commit and
-- the in-proc publish would strand goods (a rejected lot released, or a good lot never freed). Staging it in
-- the SAME tx as the inspection makes it survive the crash. Standard 11-column outbox shape.
CREATE TABLE IF NOT EXISTS quality.outbox_events (
  id uuid PRIMARY KEY, event_type text NOT NULL, aggregate_type text NOT NULL, aggregate_id text NOT NULL,
  payload jsonb NOT NULL, occurred_at timestamptz NOT NULL, correlation_id text, causation_id text,
  version int NOT NULL DEFAULT 1, created_at timestamptz NOT NULL DEFAULT now(), published_at timestamptz );
CREATE INDEX IF NOT EXISTS idx_quality_outbox_unpublished ON quality.outbox_events (occurred_at) WHERE published_at IS NULL;
CREATE TABLE IF NOT EXISTS quality.inbox_consumed (
  consumer text NOT NULL, event_id uuid NOT NULL, consumed_at timestamptz NOT NULL DEFAULT now(), PRIMARY KEY (consumer, event_id) );
