-- Down: drop quality.quality_inspections table
DROP TABLE IF EXISTS quality.quality_inspections CASCADE;
DROP FUNCTION IF EXISTS quality.quality_inspections_audit_timestamp() CASCADE;
