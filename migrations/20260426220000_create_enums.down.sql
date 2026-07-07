-- Down: drop enum types for quality module
DROP TYPE IF EXISTS reading_result CASCADE;
DROP TYPE IF EXISTS inspection_status CASCADE;
DROP TYPE IF EXISTS inspection_type CASCADE;
DROP TYPE IF EXISTS quality_action_status CASCADE;
DROP TYPE IF EXISTS quality_action_type CASCADE;
DROP TYPE IF EXISTS non_conformance_status CASCADE;
DROP TYPE IF EXISTS non_conformance_severity CASCADE;
