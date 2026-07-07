-- Down: drop quality.quality_inspection_templates table
DROP TABLE IF EXISTS quality.quality_inspection_templates CASCADE;
DROP FUNCTION IF EXISTS quality.quality_inspection_templates_audit_timestamp() CASCADE;
