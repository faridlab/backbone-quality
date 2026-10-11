// This directory's extension (hand-written; ADR-0031). The generated mod.rs (lib.rs for
// src/) pulls it in with include!, so its lines resolve against this directory.

// The hand-written quality SQL's parameter/projection types (see the repositories declared
// `user_owned` in metaphor.codegen.yaml).
pub use non_conformance_repository::{LockedNonConformanceRow, NewNonConformanceRow};
pub use quality_action_repository::NewQualityActionRow;
pub use quality_inspection_parameter_repository::{CriterionRow, NewTemplateParameterRow};
pub use quality_inspection_reading_repository::NewReadingRow;
pub use quality_inspection_repository::NewInspectionRow;
pub use quality_inspection_template_repository::NewTemplateRow;
pub use quality_procedure_repository::NewProcedureRow;
