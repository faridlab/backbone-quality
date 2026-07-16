-- Down: remove the company RLS fence for quality module

-- Reverse the company RLS fence for quality.non_conformances
DROP POLICY IF EXISTS non_conformances_company_isolation ON quality.non_conformances;
ALTER TABLE quality.non_conformances NO FORCE ROW LEVEL SECURITY;
ALTER TABLE quality.non_conformances DISABLE ROW LEVEL SECURITY;

-- Reverse the company RLS fence for quality.quality_actions
DROP POLICY IF EXISTS quality_actions_company_isolation ON quality.quality_actions;
ALTER TABLE quality.quality_actions NO FORCE ROW LEVEL SECURITY;
ALTER TABLE quality.quality_actions DISABLE ROW LEVEL SECURITY;

-- Reverse the company RLS fence for quality.quality_inspections
DROP POLICY IF EXISTS quality_inspections_company_isolation ON quality.quality_inspections;
ALTER TABLE quality.quality_inspections NO FORCE ROW LEVEL SECURITY;
ALTER TABLE quality.quality_inspections DISABLE ROW LEVEL SECURITY;

-- Reverse the company RLS fence for quality.quality_inspection_templates
DROP POLICY IF EXISTS quality_inspection_templates_company_isolation ON quality.quality_inspection_templates;
ALTER TABLE quality.quality_inspection_templates NO FORCE ROW LEVEL SECURITY;
ALTER TABLE quality.quality_inspection_templates DISABLE ROW LEVEL SECURITY;

-- Reverse the company RLS fence for quality.quality_procedures
DROP POLICY IF EXISTS quality_procedures_company_isolation ON quality.quality_procedures;
ALTER TABLE quality.quality_procedures NO FORCE ROW LEVEL SECURITY;
ALTER TABLE quality.quality_procedures DISABLE ROW LEVEL SECURITY;

