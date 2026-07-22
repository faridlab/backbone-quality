DROP POLICY IF EXISTS outbox_events_company_isolation ON quality.outbox_events;
ALTER TABLE quality.outbox_events NO FORCE ROW LEVEL SECURITY;
ALTER TABLE quality.outbox_events DISABLE ROW LEVEL SECURITY;
DROP INDEX IF EXISTS quality.idx_quality_outbox_company_id;
ALTER TABLE quality.outbox_events DROP COLUMN IF EXISTS company_id;
