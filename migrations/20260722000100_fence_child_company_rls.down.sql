-- Down: reverse ADR-0010 Decision A for the two child tables — un-fence, drop the company_id
-- index, make the column nullable again, then drop it. Order matters: the FORCE/ENABLE and the
-- policy must go before the column can be touched (FORCE RLS otherwise denies this script's own
-- writes under a non-empty app.company_id setting).

-- =============================================================================
-- quality_inspection_parameters
-- =============================================================================
DROP POLICY IF EXISTS quality_inspection_parameters_company_isolation ON quality.quality_inspection_parameters;
ALTER TABLE quality.quality_inspection_parameters NO FORCE ROW LEVEL SECURITY;
ALTER TABLE quality.quality_inspection_parameters DISABLE ROW LEVEL SECURITY;

DROP INDEX IF EXISTS quality.idx_quality_inspection_parameters_company_id;
ALTER TABLE quality.quality_inspection_parameters
    ALTER COLUMN company_id DROP NOT NULL;
ALTER TABLE quality.quality_inspection_parameters
    DROP COLUMN IF EXISTS company_id;

-- =============================================================================
-- quality_inspection_readings
-- =============================================================================
DROP POLICY IF EXISTS quality_inspection_readings_company_isolation ON quality.quality_inspection_readings;
ALTER TABLE quality.quality_inspection_readings NO FORCE ROW LEVEL SECURITY;
ALTER TABLE quality.quality_inspection_readings DISABLE ROW LEVEL SECURITY;

DROP INDEX IF EXISTS quality.idx_quality_inspection_readings_company_id;
ALTER TABLE quality.quality_inspection_readings
    ALTER COLUMN company_id DROP NOT NULL;
ALTER TABLE quality.quality_inspection_readings
    DROP COLUMN IF EXISTS company_id;
