-- Down: drop quality.quality_inspection_parameters table
DROP TABLE IF EXISTS quality.quality_inspection_parameters CASCADE;
DROP FUNCTION IF EXISTS quality.quality_inspection_parameters_audit_timestamp() CASCADE;
