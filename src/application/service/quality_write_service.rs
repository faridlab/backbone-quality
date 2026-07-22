//! The hand-authored quality write path (user-owned; survives regen).
//!
//! An inspection + CAPA engine. Posts NO GL and drives no neighbour. The load-bearing logic is the
//! **inspection verdict**: each reading is judged against its parameter's snapshotted criterion (numeric
//! [min,max] range, or a manual pass), and the inspection is ACCEPTED iff every reading is accepted, else
//! REJECTED. The disposition is published as an event Stock subscribes to (brief §5.3). A rejected
//! inspection can raise a NonConformance, which gathers corrective/preventive actions (CAPA) and closes
//! only once its actions are completed.
//!
//! Verdict/lifecycle timestamps are passed in (`now`) so the logic is deterministic under test.

use backbone_orm::company_scope;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::infrastructure::persistence::{
    NewInspectionRow, NewNonConformanceRow, NewProcedureRow, NewQualityActionRow, NewReadingRow,
    NewTemplateParameterRow, NewTemplateRow, NonConformanceRepository, QualityActionRepository,
    QualityInspectionParameterRepository, QualityInspectionReadingRepository,
    QualityInspectionRepository, QualityInspectionTemplateRepository, QualityProcedureRepository,
};

use super::quality_events::*;

#[derive(Debug, thiserror::Error)]
pub enum QualityError {
    #[error("db: {0}")]
    Db(#[from] sqlx::Error),
    #[error("not found: {0}")]
    NotFound(&'static str),
    #[error("invalid state: {0}")]
    InvalidState(&'static str),
    #[error("invalid input: {0}")]
    Invalid(String),
}

pub struct NewTemplateParameter {
    pub parameter_name: String,
    pub numeric: bool,
    pub min_value: Option<Decimal>,
    pub max_value: Option<Decimal>,
    pub spec_text: Option<String>,
}
pub struct NewTemplate {
    pub company_id: Uuid,
    pub template_name: String,
    pub item_id: Option<Uuid>,
    pub parameters: Vec<NewTemplateParameter>,
}

pub struct NewProcedure {
    pub company_id: Uuid,
    pub procedure_name: String,
    pub parent_procedure_id: Option<Uuid>,
    pub description: Option<String>,
}

pub struct NewReading {
    pub parameter_name: String,
    pub value: Option<Decimal>,
    pub manual_pass: Option<bool>,
    pub remarks: Option<String>,
}
pub struct NewInspection {
    pub company_id: Uuid,
    pub template_id: Uuid,
    pub item_id: Uuid,
    pub inspection_type: String, // inspection_type variant
    pub source_type: Option<String>,
    pub source_id: Option<Uuid>,
    pub sample_size: i32,
    pub readings: Vec<NewReading>,
}

pub struct NewNonConformance {
    pub company_id: Uuid,
    pub subject: String,
    pub source_inspection_id: Option<Uuid>,
    pub item_id: Option<Uuid>,
    pub severity: String, // non_conformance_severity variant
    pub description: Option<String>,
}

pub struct NewQualityAction {
    pub non_conformance_id: Uuid,
    pub action_type: String, // quality_action_type variant
    pub procedure_id: Option<Uuid>,
    pub description: String,
    pub due_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InspectOutcome {
    pub inspection_id: Uuid,
    pub accepted: bool,
}

pub struct QualityWriteService {
    pool: PgPool,
    templates: QualityInspectionTemplateRepository,
    template_parameters: QualityInspectionParameterRepository,
    procedures: QualityProcedureRepository,
    inspections: QualityInspectionRepository,
    readings: QualityInspectionReadingRepository,
    non_conformances: NonConformanceRepository,
    actions: QualityActionRepository,
}

impl QualityWriteService {
    pub fn new(pool: PgPool) -> Self {
        let templates = QualityInspectionTemplateRepository::new(pool.clone());
        let template_parameters = QualityInspectionParameterRepository::new(pool.clone());
        let procedures = QualityProcedureRepository::new(pool.clone());
        let inspections = QualityInspectionRepository::new(pool.clone());
        let readings = QualityInspectionReadingRepository::new(pool.clone());
        let non_conformances = NonConformanceRepository::new(pool.clone());
        let actions = QualityActionRepository::new(pool.clone());
        Self {
            pool, templates, template_parameters, procedures, inspections, readings,
            non_conformances, actions,
        }
    }

    /// Define an inspection template with its parameters + acceptance criteria. Each numeric parameter
    /// needs a coherent range (min <= max when both given); at least one parameter is required.
    pub async fn create_template(&self, t: NewTemplate) -> Result<Uuid, QualityError> {
        if t.template_name.trim().is_empty() {
            return Err(QualityError::Invalid("template needs a name".into()));
        }
        if t.parameters.is_empty() {
            return Err(QualityError::Invalid("a template needs at least one parameter".into()));
        }
        for p in &t.parameters {
            if p.numeric {
                if let (Some(lo), Some(hi)) = (p.min_value, p.max_value) {
                    if lo > hi {
                        return Err(QualityError::Invalid("parameter min must be <= max".into()));
                    }
                }
                if p.min_value.is_none() && p.max_value.is_none() {
                    return Err(QualityError::Invalid("a numeric parameter needs a min and/or max bound".into()));
                }
            }
        }
        let id = Uuid::new_v4();
        let mut tx = self.pool.begin().await?;
        // RLS scope (ADR-0008): company is on the DTO — bind it explicitly onto this transaction, so the
        // template + parameter inserts pass the `app.company_id` fence.
        company_scope::bind_company_on(&mut tx, t.company_id).await?;
        self.templates.insert_template(&mut tx, &NewTemplateRow {
            id,
            company_id: t.company_id,
            template_name: &t.template_name,
            item_id: t.item_id,
        }).await?;
        for p in &t.parameters {
            self.template_parameters.insert_parameter(&mut tx, &NewTemplateParameterRow {
                id: Uuid::new_v4(),
                company_id: t.company_id,
                template_id: id,
                parameter_name: &p.parameter_name,
                numeric: p.numeric,
                min_value: p.min_value,
                max_value: p.max_value,
                spec_text: p.spec_text.as_deref(),
            }).await?;
        }
        tx.commit().await?;
        Ok(id)
    }

    /// Define a quality procedure (adjacency-list tree). A parent, if given, must be in the same company.
    pub async fn create_procedure(&self, p: NewProcedure) -> Result<Uuid, QualityError> {
        if p.procedure_name.trim().is_empty() {
            return Err(QualityError::Invalid("procedure needs a name".into()));
        }
        // RLS scope (ADR-0008): company is on the DTO — scope each query to it. The explicit
        // `company_id = $2` filter below stays as defense-in-depth.
        if let Some(parent) = p.parent_procedure_id {
            let ok: Option<Uuid> = company_scope::with_company_scope(
                Some(p.company_id),
                self.procedures.find_id_in_company(&self.pool, parent, p.company_id),
            ).await?;
            if ok.is_none() {
                return Err(QualityError::Invalid("parent procedure is not in this company".into()));
            }
        }
        let id = Uuid::new_v4();
        let row = NewProcedureRow {
            id,
            company_id: p.company_id,
            procedure_name: &p.procedure_name,
            parent_procedure_id: p.parent_procedure_id,
            description: p.description.as_deref(),
        };
        company_scope::with_company_scope(
            Some(p.company_id),
            self.procedures.insert_procedure(&self.pool, &row),
        ).await?;
        Ok(id)
    }

    /// Inspect an item against a template — THE verdict. Each reading is judged against its parameter's
    /// criterion (numeric: value within [min,max]; non-numeric: a manual pass), the criterion is
    /// snapshotted onto the reading, and the inspection is ACCEPTED iff every reading is accepted. Emits
    /// `QualityInspectionCompleted` (the disposition Stock subscribes to).
    pub async fn inspect(
        &self,
        i: NewInspection,
        now: DateTime<Utc>,
        sink: &dyn QualityEventSink,
    ) -> Result<InspectOutcome, QualityError> {
        if i.readings.is_empty() {
            return Err(QualityError::Invalid("an inspection needs at least one reading".into()));
        }
        // Load the WHOLE template's criteria in one snapshot (consistent across readings, and the basis
        // for the coverage check below). An `accepted` verdict must mean the item conforms to the
        // TEMPLATE — so every template parameter must be measured exactly once. Judging only the
        // caller-supplied readings would let a truncated readings set (dropping the parameter that would
        // fail) or a concurrently-added parameter pass unmeasured (maturity council 2026-07-07).
        // RLS scope (ADR-0008): company is on the DTO — scope the criteria snapshot to it, so a template
        // belonging to another tenant reads as absent (`NotFound`) rather than leaking its criteria.
        let params = company_scope::with_company_scope(
            Some(i.company_id),
            self.template_parameters.list_criteria(&self.pool, i.template_id),
        ).await?;
        if params.is_empty() {
            return Err(QualityError::NotFound("template"));
        }
        use std::collections::HashMap;
        let mut criteria: HashMap<String, (bool, Option<Decimal>, Option<Decimal>)> = HashMap::new();
        for p in &params {
            criteria.insert(p.parameter_name.clone(), (p.numeric, p.min_value, p.max_value));
        }
        // Coverage: exactly one reading per template parameter — no unknown, no duplicate, none missing.
        let mut seen: std::collections::HashSet<&str> = std::collections::HashSet::new();
        for r in &i.readings {
            if !criteria.contains_key(&r.parameter_name) {
                return Err(QualityError::Invalid(format!("parameter '{}' is not in the template", r.parameter_name)));
            }
            if !seen.insert(r.parameter_name.as_str()) {
                return Err(QualityError::Invalid(format!("duplicate reading for parameter '{}'", r.parameter_name)));
            }
        }
        if let Some(missing) = criteria.keys().find(|k| !seen.contains(k.as_str())) {
            return Err(QualityError::Invalid(format!("no reading for template parameter '{missing}'")));
        }

        // Judge each reading against its parameter's criterion (from the single snapshot above).
        struct Judged {
            parameter_name: String,
            numeric: bool,
            value: Option<Decimal>,
            min_value: Option<Decimal>,
            max_value: Option<Decimal>,
            manual_result: Option<bool>,
            accepted: bool,
            remarks: Option<String>,
        }
        let mut judged = Vec::with_capacity(i.readings.len());
        let mut all_accepted = true;
        for r in &i.readings {
            let (numeric, min_value, max_value) = criteria[&r.parameter_name];
            let accepted = if numeric {
                match r.value {
                    None => false, // a numeric parameter with no measured value cannot be accepted
                    Some(v) => min_value.map_or(true, |lo| v >= lo) && max_value.map_or(true, |hi| v <= hi),
                }
            } else {
                r.manual_pass == Some(true)
            };
            if !accepted {
                all_accepted = false;
            }
            judged.push(Judged {
                parameter_name: r.parameter_name.clone(), numeric, value: r.value,
                min_value, max_value, manual_result: r.manual_pass, accepted, remarks: r.remarks.clone(),
            });
        }
        let status = if all_accepted { "accepted" } else { "rejected" };

        let id = Uuid::new_v4();
        let mut tx = self.pool.begin().await?;
        // RLS scope (ADR-0008): bind this tx to the inspection's company, so the inspection + readings
        // inserts and the outbox stage all pass the fence.
        company_scope::bind_company_on(&mut tx, i.company_id).await?;
        self.inspections.insert_inspection(&mut tx, &NewInspectionRow {
            id,
            company_id: i.company_id,
            template_id: i.template_id,
            item_id: i.item_id,
            inspection_type: &i.inspection_type,
            source_type: i.source_type.as_deref(),
            source_id: i.source_id,
            sample_size: i.sample_size,
            inspected_at: now,
            status,
        }).await?;
        for j in &judged {
            let result = if j.accepted { "accepted" } else { "rejected" };
            self.readings.insert_reading(&mut tx, &NewReadingRow {
                id: Uuid::new_v4(),
                company_id: i.company_id,
                inspection_id: id,
                parameter_name: &j.parameter_name,
                numeric: j.numeric,
                reading_value: j.value,
                min_value: j.min_value,
                max_value: j.max_value,
                manual_result: j.manual_result,
                result,
                remarks: j.remarks.as_deref(),
            }).await?;
        }
        let accepted = all_accepted;
        let event = QualityEvent::QualityInspectionCompleted(QualityInspectionCompleted {
            inspection_id: id, company_id: i.company_id, item_id: i.item_id,
            inspection_type: i.inspection_type, source_type: i.source_type, source_id: i.source_id, accepted,
        });
        // Stage the disposition durably in the SAME tx as the inspection (outbox rollout plan, P1): Stock
        // subscribes to it to accept/reject the lot, so a crash between commit and the in-proc publish must
        // not drop it. Then commit, then publish in-proc (the fast path; the relay drains the outbox).
        let record = backbone_outbox::OutboxRecord::new(
            "QualityInspectionCompleted", "QualityInspection", id.to_string(), i.company_id,
            serde_json::to_value(&event).map_err(|e| QualityError::Invalid(e.to_string()))?,
            chrono::Utc::now(),
        );
        backbone_outbox::outbox::stage(&mut *tx, "quality", &record)
            .await.map_err(|e| QualityError::Invalid(format!("outbox stage: {e}")))?;
        tx.commit().await?;
        sink.publish(&event);
        Ok(InspectOutcome { inspection_id: id, accepted })
    }

    /// Raise a non-conformance. If it cites a source inspection, that inspection must be REJECTED (you
    /// don't raise an NC against a passing inspection). Emits `NonConformanceRaised`.
    pub async fn raise_non_conformance(
        &self,
        nc: NewNonConformance,
        now: DateTime<Utc>,
        sink: &dyn QualityEventSink,
    ) -> Result<Uuid, QualityError> {
        if nc.subject.trim().is_empty() {
            return Err(QualityError::Invalid("non-conformance needs a subject".into()));
        }
        // RLS scope (ADR-0008): company is on the DTO — scope both the source-inspection check and the
        // insert to it. The cited inspection must be visible in THIS company or it reads as not found.
        if let Some(insp) = nc.source_inspection_id {
            let st: Option<String> = company_scope::with_company_scope(
                Some(nc.company_id),
                self.inspections.find_status(&self.pool, insp),
            ).await?;
            match st.as_deref() {
                None => return Err(QualityError::NotFound("source inspection")),
                Some("rejected") => {}
                Some(_) => return Err(QualityError::InvalidState("source inspection was not rejected")),
            }
        }
        let id = Uuid::new_v4();
        let row = NewNonConformanceRow {
            id,
            company_id: nc.company_id,
            subject: &nc.subject,
            source_inspection_id: nc.source_inspection_id,
            item_id: nc.item_id,
            severity: &nc.severity,
            description: nc.description.as_deref(),
            opened_at: now,
        };
        company_scope::with_company_scope(
            Some(nc.company_id),
            self.non_conformances.insert_non_conformance(&self.pool, &row),
        ).await?;
        sink.publish(&QualityEvent::NonConformanceRaised(NonConformanceRaised {
            non_conformance_id: id, company_id: nc.company_id,
            source_inspection_id: nc.source_inspection_id, severity: nc.severity,
        }));
        Ok(id)
    }

    /// Add a corrective/preventive action to an open non-conformance, advancing it to `in_progress`. The
    /// NC row is locked for the duration so a concurrent close can't slip a fresh action past it. A cited
    /// procedure must belong to the SAME company as the NC (ADR-0010 F2 — the procedure_id was previously
    /// inserted unvalidated, so a CAPA could cite a foreign-company procedure).
    pub async fn add_quality_action(&self, a: NewQualityAction) -> Result<Uuid, QualityError> {
        if a.description.trim().is_empty() {
            return Err(QualityError::Invalid("action needs a description".into()));
        }
        let mut tx = self.pool.begin().await?;
        // RLS scope (ADR-0008), ID-only pattern: this method carries NO company — it is identified by the
        // non-conformance id alone, and the action's company is derived by sub-SELECT from the NC row. So
        // bind the AMBIENT task-local scope onto the tx: under HTTP that is the caller's company (set by
        // `company_auth`), so the locking read below cannot see another tenant's NC. A non-HTTP CALLER
        // (job/event driver) MUST wrap this call in `with_company_scope(Some(company_id))` or it fails closed.
        company_scope::bind_current_company(&mut tx).await?;
        let row = self.non_conformances.lock_status(&mut tx, a.non_conformance_id).await?;
        let nc = match row {
            None => { tx.rollback().await?; return Err(QualityError::NotFound("non-conformance")); }
            Some(r) => r,
        };
        if nc.status == "closed" {
            tx.rollback().await?;
            return Err(QualityError::InvalidState("non-conformance is closed"));
        }
        // F2 (ADR-0010): a cited procedure must belong to the NC's company. The NC's company is the only
        // company we can trust here (it is what the action's company will be sub-SELECTed from at insert),
        // so validate against IT — not the ambient scope, which is only equal to it under HTTP. The probe
        // runs on the pool (a separate connection from the tx) wrapped in `with_company_scope` so the
        // procedures' RLS fence admits the row; the explicit `company_id = $2` filter stays as
        // defense-in-depth. A foreign or missing procedure reads as None → Invalid.
        if let Some(procedure_id) = a.procedure_id {
            let ok: Option<Uuid> = company_scope::with_company_scope(
                Some(nc.company_id),
                self.procedures.find_id_in_company(&self.pool, procedure_id, nc.company_id),
            ).await?;
            if ok.is_none() {
                tx.rollback().await?;
                return Err(QualityError::Invalid("cited procedure is not in this company".into()));
            }
        }
        let id = Uuid::new_v4();
        self.actions.insert_action(&mut tx, &NewQualityActionRow {
            id,
            non_conformance_id: a.non_conformance_id,
            action_type: &a.action_type,
            procedure_id: a.procedure_id,
            description: &a.description,
            due_date: a.due_date,
        }).await?;
        self.non_conformances.mark_in_progress(&mut tx, a.non_conformance_id).await?;
        tx.commit().await?;
        Ok(id)
    }

    /// Mark an action completed (terminal). Idempotent — a re-complete is a no-op.
    pub async fn complete_action(&self, action_id: Uuid, now: DateTime<Utc>) -> Result<(), QualityError> {
        // RLS scope (ADR-0008), ID-only pattern: identified by the action id alone, with no company to
        // scope from. Both statements ride the REQUEST-dedicated connection, which carries the caller's
        // `app.company_id` — another tenant's action is simply not found. A non-HTTP CALLER must wrap this
        // in `with_company_scope(Some(company_id))`.
        let moved = self.actions.complete(&self.pool, action_id, now).await?;
        if moved == 0 {
            // Either already completed (idempotent no-op) or not found.
            let exists: Option<Uuid> = self.actions.exists(&self.pool, action_id).await?;
            if exists.is_none() {
                return Err(QualityError::NotFound("action"));
            }
        }
        Ok(())
    }

    /// Close a non-conformance — only once every action is completed. The NC row is locked so a
    /// concurrent `add_quality_action` serializes: it either lands before the close (blocking it) or sees
    /// the closed status and is refused. Emits `NonConformanceClosed`.
    pub async fn close_non_conformance(
        &self,
        nc_id: Uuid,
        now: DateTime<Utc>,
        sink: &dyn QualityEventSink,
    ) -> Result<(), QualityError> {
        let mut tx = self.pool.begin().await?;
        // RLS scope (ADR-0008), ID-only pattern: identified by the NC id alone — the company is only known
        // AFTER the locking read below, so it cannot be bound up front. Bind the AMBIENT task-local scope
        // (the caller's company under HTTP) so that read is fenced; a non-HTTP CALLER MUST wrap this call
        // in `with_company_scope(Some(company_id))` or it fails closed.
        company_scope::bind_current_company(&mut tx).await?;
        let row = self.non_conformances.lock_for_close(&mut tx, nc_id).await?;
        let row = match row {
            Some(r) => r,
            None => { tx.rollback().await?; return Err(QualityError::NotFound("non-conformance")); }
        };
        if row.status == "closed" {
            tx.rollback().await?;
            return Ok(()); // idempotent
        }
        let open_actions: i64 = self.actions.count_incomplete(&mut tx, nc_id).await?;
        if open_actions > 0 {
            tx.rollback().await?;
            return Err(QualityError::InvalidState("cannot close — incomplete actions remain"));
        }
        let company_id: Uuid = row.company_id;
        self.non_conformances.close(&mut tx, nc_id, now).await?;
        tx.commit().await?;
        sink.publish(&QualityEvent::NonConformanceClosed(NonConformanceClosed { non_conformance_id: nc_id, company_id }));
        Ok(())
    }
}
