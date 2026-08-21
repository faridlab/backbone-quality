-- Migration: replace the two quality lifecycle booleans with status enums
-- quality_procedures and quality_inspection_templates each carried
-- `is_active BOOLEAN NOT NULL DEFAULT TRUE`; the tree-wide convention is one
-- `status` enum field per lifecycle (see docs/refactoring-schema in the serpa
-- workspace). Each boolean migrates only rows deviating from its own column
-- default. The enum types are created unqualified so they land beside the
-- module's other enum types (public), where the generated sqlx type_name
-- resolves. The old is_active indexes die with their column; status indexes
-- take their place.

DO $$ BEGIN
    CREATE TYPE quality_procedure_status AS ENUM ('active', 'inactive');
EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN
    CREATE TYPE quality_inspection_template_status AS ENUM ('active', 'inactive');
EXCEPTION WHEN duplicate_object THEN NULL; END $$;

ALTER TABLE quality.quality_procedures ADD COLUMN status quality_procedure_status NOT NULL DEFAULT 'active';
UPDATE quality.quality_procedures SET status = 'inactive' WHERE NOT is_active;
ALTER TABLE quality.quality_procedures DROP COLUMN is_active;

ALTER TABLE quality.quality_inspection_templates ADD COLUMN status quality_inspection_template_status NOT NULL DEFAULT 'active';
UPDATE quality.quality_inspection_templates SET status = 'inactive' WHERE NOT is_active;
ALTER TABLE quality.quality_inspection_templates DROP COLUMN is_active;

CREATE INDEX IF NOT EXISTS idx_quality_procedures_company_id_status ON quality.quality_procedures (company_id, status);
CREATE INDEX IF NOT EXISTS idx_quality_inspection_templates_company_id_status ON quality.quality_inspection_templates (company_id, status);
