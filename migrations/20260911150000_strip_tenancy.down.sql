-- Hand-authored (user-owned). Not regenerated.
--
-- Best-effort restore sketch for the tenancy strip (ADR-0029). This is a breaking module
-- release against dev-stage databases: the down re-adds the company_id column as nullable
-- with its plain indexes and the company isolation policy shape, but restores NO data —
-- rows written after the strip (or after the decorator re-keyed them) carry org_unit_id
-- only. The composing service's tenancy decorator remains the live fence; treat this
-- down as a schema-shape sketch for archaeology, not a usable rollback.
--
-- quality.outbox_events keeps its company_id column (the strip never dropped it — the
-- framework's stage() requires it); only its index and isolation policy are re-created.

ALTER TABLE quality.non_conformances             ADD COLUMN IF NOT EXISTS company_id uuid;
ALTER TABLE quality.quality_actions              ADD COLUMN IF NOT EXISTS company_id uuid;
ALTER TABLE quality.quality_inspections          ADD COLUMN IF NOT EXISTS company_id uuid;
ALTER TABLE quality.quality_inspection_readings  ADD COLUMN IF NOT EXISTS company_id uuid;
ALTER TABLE quality.quality_inspection_parameters ADD COLUMN IF NOT EXISTS company_id uuid;
ALTER TABLE quality.quality_inspection_templates ADD COLUMN IF NOT EXISTS company_id uuid;
ALTER TABLE quality.quality_procedures           ADD COLUMN IF NOT EXISTS company_id uuid;

CREATE INDEX IF NOT EXISTS idx_non_conformances_company_id_status
    ON quality.non_conformances (company_id, status);
CREATE INDEX IF NOT EXISTS idx_quality_inspections_company_id_status
    ON quality.quality_inspections (company_id, status);
CREATE INDEX IF NOT EXISTS idx_quality_inspection_readings_company_id
    ON quality.quality_inspection_readings (company_id);
CREATE INDEX IF NOT EXISTS idx_quality_inspection_parameters_company_id
    ON quality.quality_inspection_parameters (company_id);
CREATE INDEX IF NOT EXISTS idx_quality_inspection_templates_company_id_status
    ON quality.quality_inspection_templates (company_id, status);
CREATE INDEX IF NOT EXISTS idx_quality_procedures_company_id_status
    ON quality.quality_procedures (company_id, status);
CREATE INDEX IF NOT EXISTS idx_quality_outbox_company_id
    ON quality.outbox_events (company_id);

CREATE POLICY non_conformances_company_isolation ON quality.non_conformances
    FOR ALL USING (company_id = NULLIF(current_setting('app.company_id', true), '')::uuid)
    WITH CHECK (company_id = NULLIF(current_setting('app.company_id', true), '')::uuid);
CREATE POLICY quality_actions_company_isolation ON quality.quality_actions
    FOR ALL USING (company_id = NULLIF(current_setting('app.company_id', true), '')::uuid)
    WITH CHECK (company_id = NULLIF(current_setting('app.company_id', true), '')::uuid);
CREATE POLICY quality_inspections_company_isolation ON quality.quality_inspections
    FOR ALL USING (company_id = NULLIF(current_setting('app.company_id', true), '')::uuid)
    WITH CHECK (company_id = NULLIF(current_setting('app.company_id', true), '')::uuid);
CREATE POLICY quality_inspection_readings_company_isolation ON quality.quality_inspection_readings
    FOR ALL USING (company_id = NULLIF(current_setting('app.company_id', true), '')::uuid)
    WITH CHECK (company_id = NULLIF(current_setting('app.company_id', true), '')::uuid);
CREATE POLICY quality_inspection_parameters_company_isolation ON quality.quality_inspection_parameters
    FOR ALL USING (company_id = NULLIF(current_setting('app.company_id', true), '')::uuid)
    WITH CHECK (company_id = NULLIF(current_setting('app.company_id', true), '')::uuid);
CREATE POLICY quality_inspection_templates_company_isolation ON quality.quality_inspection_templates
    FOR ALL USING (company_id = NULLIF(current_setting('app.company_id', true), '')::uuid)
    WITH CHECK (company_id = NULLIF(current_setting('app.company_id', true), '')::uuid);
CREATE POLICY quality_procedures_company_isolation ON quality.quality_procedures
    FOR ALL USING (company_id = NULLIF(current_setting('app.company_id', true), '')::uuid)
    WITH CHECK (company_id = NULLIF(current_setting('app.company_id', true), '')::uuid);
CREATE POLICY outbox_events_company_isolation ON quality.outbox_events
    FOR ALL USING (company_id = NULLIF(current_setting('app.company_id', true), '')::uuid)
    WITH CHECK (company_id = NULLIF(current_setting('app.company_id', true), '')::uuid);
