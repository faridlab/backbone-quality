-- Down: restore the two is_active booleans exactly as they were.
-- Only 'inactive' rows are written back as FALSE; rows at the column default
-- map to the boolean default TRUE without an UPDATE. The status indexes die
-- with their column; the original is_active indexes are recreated by name.

ALTER TABLE quality.quality_procedures ADD COLUMN is_active BOOLEAN NOT NULL DEFAULT TRUE;
UPDATE quality.quality_procedures SET is_active = FALSE WHERE status = 'inactive';
ALTER TABLE quality.quality_procedures DROP COLUMN status;

ALTER TABLE quality.quality_inspection_templates ADD COLUMN is_active BOOLEAN NOT NULL DEFAULT TRUE;
UPDATE quality.quality_inspection_templates SET is_active = FALSE WHERE status = 'inactive';
ALTER TABLE quality.quality_inspection_templates DROP COLUMN status;

CREATE INDEX IF NOT EXISTS idx_quality_procedures_company_id_is_active ON quality.quality_procedures (company_id, is_active);
CREATE INDEX IF NOT EXISTS idx_quality_inspection_templates_company_id_is_active ON quality.quality_inspection_templates (company_id, is_active);

DROP TYPE IF EXISTS quality_procedure_status;
DROP TYPE IF EXISTS quality_inspection_template_status;
