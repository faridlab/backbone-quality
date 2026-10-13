-- Down: drop the declared foreign keys for quality module
ALTER TABLE IF EXISTS quality.quality_procedures DROP CONSTRAINT IF EXISTS fk_quality_procedures_parent_procedure_id;
ALTER TABLE IF EXISTS quality.quality_inspection_parameters DROP CONSTRAINT IF EXISTS fk_quality_inspection_parameters_template_id;
ALTER TABLE IF EXISTS quality.quality_inspection_readings DROP CONSTRAINT IF EXISTS fk_quality_inspection_readings_inspection_id;
ALTER TABLE IF EXISTS quality.quality_actions DROP CONSTRAINT IF EXISTS fk_quality_actions_non_conformance_id;
