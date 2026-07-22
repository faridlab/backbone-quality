-- ADR-0011: fence quality.outbox_events by company_id (extracted from the payload).
ALTER TABLE quality.outbox_events ADD COLUMN IF NOT EXISTS company_id UUID;
UPDATE quality.outbox_events SET company_id = (payload ->> 'company_id')::uuid WHERE company_id IS NULL;
ALTER TABLE quality.outbox_events ALTER COLUMN company_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_quality_outbox_company_id ON quality.outbox_events (company_id);
ALTER TABLE quality.outbox_events ENABLE ROW LEVEL SECURITY;
ALTER TABLE quality.outbox_events FORCE  ROW LEVEL SECURITY;
DROP POLICY IF EXISTS outbox_events_company_isolation ON quality.outbox_events;
CREATE POLICY outbox_events_company_isolation ON quality.outbox_events
    FOR ALL
    USING      (company_id = NULLIF(current_setting('app.company_id', true), '')::uuid)
    WITH CHECK (company_id = NULLIF(current_setting('app.company_id', true), '')::uuid);
