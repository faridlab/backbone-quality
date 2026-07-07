-- Down: drop quality.quality_inspection_readings table
DROP TABLE IF EXISTS quality.quality_inspection_readings CASCADE;
DROP FUNCTION IF EXISTS quality.quality_inspection_readings_audit_timestamp() CASCADE;
