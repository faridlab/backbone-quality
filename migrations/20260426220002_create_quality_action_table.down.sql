-- Down: drop quality.quality_actions table
DROP TABLE IF EXISTS quality.quality_actions CASCADE;
DROP FUNCTION IF EXISTS quality.quality_actions_audit_timestamp() CASCADE;
