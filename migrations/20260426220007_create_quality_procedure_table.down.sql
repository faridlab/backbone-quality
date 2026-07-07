-- Down: drop quality.quality_procedures table
DROP TABLE IF EXISTS quality.quality_procedures CASCADE;
DROP FUNCTION IF EXISTS quality.quality_procedures_audit_timestamp() CASCADE;
