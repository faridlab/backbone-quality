-- Down: drop quality.non_conformances table
DROP TABLE IF EXISTS quality.non_conformances CASCADE;
DROP FUNCTION IF EXISTS quality.non_conformances_audit_timestamp() CASCADE;
