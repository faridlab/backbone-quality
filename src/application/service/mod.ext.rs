// This directory's extension (hand-written; ADR-0031). The generated mod.rs (lib.rs for
// src/) pulls it in with include!, so its lines resolve against this directory.

pub mod quality_events;
pub mod quality_write_service;
pub use quality_events::{
    LoggingSink, NonConformanceClosed, NonConformanceRaised, QualityEvent, QualityEventSink,
    QualityInspectionCompleted,
};
pub use quality_write_service::{
    InspectOutcome, NewInspection, NewNonConformance, NewProcedure, NewQualityAction, NewReading,
    NewTemplate, NewTemplateParameter, QualityError, QualityWriteService,
};
