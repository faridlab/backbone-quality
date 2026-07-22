-- ADR-0010 Decision A: extend the company RLS fence to the two child tables that were missed by
-- ADR-0008 — quality_inspection_readings and quality_inspection_parameters.
--
-- Pattern: DENORMALIZED company_id column (explicitly NOT a parent-join EXISTS subquery policy).
-- ADR-0010 rejected the parent-join as the default: it costs a correlated subquery on every read
-- and complicates WITH CHECK. Here company_id is copied from the parent at write time and the
-- FORALL policy is the same trivial expression the parents use. The parents are already fenced, so
-- the backfill is deterministic and cannot fail loud on a leaked foreign row (a foreign parent
-- reads as absent under RLS, so no reading/parameter row can reference it).
--
-- No hard SQL FK to organization.companies (logical-FK-only convention, like every other company_id
-- in this module). The child→parent FK (inspection_id / template_id) is unchanged.

-- =============================================================================
-- quality_inspection_readings: ADD company_id, backfill from parent, fence.
-- =============================================================================
ALTER TABLE quality.quality_inspection_readings
    ADD COLUMN IF NOT EXISTS company_id UUID;

UPDATE quality.quality_inspection_readings r
SET company_id = i.company_id
FROM quality.quality_inspections i
WHERE r.inspection_id = i.id
  AND r.company_id IS NULL;

-- Every reading has a parent inspection (inspection_id is NOT NULL with a hard FK), so the
-- backfill is total. Make the column NOT NULL, then index it for the RLS policy's filter.
ALTER TABLE quality.quality_inspection_readings
    ALTER COLUMN company_id SET NOT NULL;

CREATE INDEX IF NOT EXISTS idx_quality_inspection_readings_company_id
    ON quality.quality_inspection_readings (company_id);

ALTER TABLE quality.quality_inspection_readings ENABLE ROW LEVEL SECURITY;
ALTER TABLE quality.quality_inspection_readings FORCE  ROW LEVEL SECURITY;
DROP POLICY IF EXISTS quality_inspection_readings_company_isolation ON quality.quality_inspection_readings;
CREATE POLICY quality_inspection_readings_company_isolation ON quality.quality_inspection_readings
    FOR ALL
    USING      (company_id = NULLIF(current_setting('app.company_id', true), '')::uuid)
    WITH CHECK (company_id = NULLIF(current_setting('app.company_id', true), '')::uuid);

-- =============================================================================
-- quality_inspection_parameters: ADD company_id, backfill from parent, fence.
-- =============================================================================
ALTER TABLE quality.quality_inspection_parameters
    ADD COLUMN IF NOT EXISTS company_id UUID;

UPDATE quality.quality_inspection_parameters p
SET company_id = t.company_id
FROM quality.quality_inspection_templates t
WHERE p.template_id = t.id
  AND p.company_id IS NULL;

-- Every parameter has a parent template (template_id is NOT NULL with a hard FK), so the backfill
-- is total. Make the column NOT NULL, then index it for the RLS policy's filter.
ALTER TABLE quality.quality_inspection_parameters
    ALTER COLUMN company_id SET NOT NULL;

CREATE INDEX IF NOT EXISTS idx_quality_inspection_parameters_company_id
    ON quality.quality_inspection_parameters (company_id);

ALTER TABLE quality.quality_inspection_parameters ENABLE ROW LEVEL SECURITY;
ALTER TABLE quality.quality_inspection_parameters FORCE  ROW LEVEL SECURITY;
DROP POLICY IF EXISTS quality_inspection_parameters_company_isolation ON quality.quality_inspection_parameters;
CREATE POLICY quality_inspection_parameters_company_isolation ON quality.quality_inspection_parameters
    FOR ALL
    USING      (company_id = NULLIF(current_setting('app.company_id', true), '')::uuid)
    WITH CHECK (company_id = NULLIF(current_setting('app.company_id', true), '')::uuid);
