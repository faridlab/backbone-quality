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

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::{PgPool, Row};
use uuid::Uuid;

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
}

impl QualityWriteService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
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
        sqlx::query(
            r#"INSERT INTO quality.quality_inspection_templates (id, company_id, template_name, item_id, is_active)
               VALUES ($1,$2,$3,$4,true)"#,
        )
        .bind(id).bind(t.company_id).bind(&t.template_name).bind(t.item_id)
        .execute(&mut *tx)
        .await?;
        for p in &t.parameters {
            sqlx::query(
                r#"INSERT INTO quality.quality_inspection_parameters
                     (id, template_id, parameter_name, numeric, min_value, max_value, spec_text)
                   VALUES ($1,$2,$3,$4,$5,$6,$7)"#,
            )
            .bind(Uuid::new_v4()).bind(id).bind(&p.parameter_name).bind(p.numeric)
            .bind(p.min_value).bind(p.max_value).bind(&p.spec_text)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        Ok(id)
    }

    /// Define a quality procedure (adjacency-list tree). A parent, if given, must be in the same company.
    pub async fn create_procedure(&self, p: NewProcedure) -> Result<Uuid, QualityError> {
        if p.procedure_name.trim().is_empty() {
            return Err(QualityError::Invalid("procedure needs a name".into()));
        }
        if let Some(parent) = p.parent_procedure_id {
            let ok: Option<Uuid> = sqlx::query_scalar(
                "SELECT id FROM quality.quality_procedures WHERE id=$1 AND company_id=$2")
                .bind(parent).bind(p.company_id).fetch_optional(&self.pool).await?;
            if ok.is_none() {
                return Err(QualityError::Invalid("parent procedure is not in this company".into()));
            }
        }
        let id = Uuid::new_v4();
        sqlx::query(
            r#"INSERT INTO quality.quality_procedures
                 (id, company_id, procedure_name, parent_procedure_id, description, is_active)
               VALUES ($1,$2,$3,$4,$5,true)"#,
        )
        .bind(id).bind(p.company_id).bind(&p.procedure_name).bind(p.parent_procedure_id).bind(&p.description)
        .execute(&self.pool)
        .await?;
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
        let params = sqlx::query(
            r#"SELECT parameter_name, numeric, min_value, max_value FROM quality.quality_inspection_parameters
               WHERE template_id=$1 AND (metadata->>'deleted_at') IS NULL"#,
        )
        .bind(i.template_id)
        .fetch_all(&self.pool)
        .await?;
        if params.is_empty() {
            return Err(QualityError::NotFound("template"));
        }
        use std::collections::HashMap;
        let mut criteria: HashMap<String, (bool, Option<Decimal>, Option<Decimal>)> = HashMap::new();
        for p in &params {
            criteria.insert(p.get("parameter_name"), (p.get("numeric"), p.get("min_value"), p.get("max_value")));
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
        sqlx::query(
            r#"INSERT INTO quality.quality_inspections
                 (id, company_id, template_id, item_id, inspection_type, source_type, source_id,
                  sample_size, inspected_at, status, remarks)
               VALUES ($1,$2,$3,$4,$5::inspection_type,$6,$7,$8,$9,$10::inspection_status,NULL)"#,
        )
        .bind(id).bind(i.company_id).bind(i.template_id).bind(i.item_id).bind(&i.inspection_type)
        .bind(&i.source_type).bind(i.source_id).bind(i.sample_size).bind(now).bind(status)
        .execute(&mut *tx)
        .await?;
        for j in &judged {
            let result = if j.accepted { "accepted" } else { "rejected" };
            sqlx::query(
                r#"INSERT INTO quality.quality_inspection_readings
                     (id, inspection_id, parameter_name, numeric, reading_value, min_value, max_value,
                      manual_result, result, remarks)
                   VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9::reading_result,$10)"#,
            )
            .bind(Uuid::new_v4()).bind(id).bind(&j.parameter_name).bind(j.numeric).bind(j.value)
            .bind(j.min_value).bind(j.max_value).bind(j.manual_result).bind(result).bind(&j.remarks)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        let accepted = all_accepted;
        sink.publish(&QualityEvent::QualityInspectionCompleted(QualityInspectionCompleted {
            inspection_id: id, company_id: i.company_id, item_id: i.item_id,
            inspection_type: i.inspection_type, source_type: i.source_type, source_id: i.source_id, accepted,
        }));
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
        if let Some(insp) = nc.source_inspection_id {
            let st: Option<String> = sqlx::query_scalar(
                "SELECT status::text FROM quality.quality_inspections WHERE id=$1 AND (metadata->>'deleted_at') IS NULL")
                .bind(insp).fetch_optional(&self.pool).await?;
            match st.as_deref() {
                None => return Err(QualityError::NotFound("source inspection")),
                Some("rejected") => {}
                Some(_) => return Err(QualityError::InvalidState("source inspection was not rejected")),
            }
        }
        let id = Uuid::new_v4();
        sqlx::query(
            r#"INSERT INTO quality.non_conformances
                 (id, company_id, subject, source_inspection_id, item_id, severity, status, description, opened_at)
               VALUES ($1,$2,$3,$4,$5,$6::non_conformance_severity,'open'::non_conformance_status,$7,$8)"#,
        )
        .bind(id).bind(nc.company_id).bind(&nc.subject).bind(nc.source_inspection_id).bind(nc.item_id)
        .bind(&nc.severity).bind(&nc.description).bind(now)
        .execute(&self.pool)
        .await?;
        sink.publish(&QualityEvent::NonConformanceRaised(NonConformanceRaised {
            non_conformance_id: id, company_id: nc.company_id,
            source_inspection_id: nc.source_inspection_id, severity: nc.severity,
        }));
        Ok(id)
    }

    /// Add a corrective/preventive action to an open non-conformance, advancing it to `in_progress`. The
    /// NC row is locked for the duration so a concurrent close can't slip a fresh action past it.
    pub async fn add_quality_action(&self, a: NewQualityAction) -> Result<Uuid, QualityError> {
        if a.description.trim().is_empty() {
            return Err(QualityError::Invalid("action needs a description".into()));
        }
        let mut tx = self.pool.begin().await?;
        let st: Option<String> = sqlx::query_scalar(
            r#"SELECT status::text FROM quality.non_conformances
               WHERE id=$1 AND (metadata->>'deleted_at') IS NULL FOR UPDATE"#,
        )
        .bind(a.non_conformance_id)
        .fetch_optional(&mut *tx)
        .await?;
        match st.as_deref() {
            None => { tx.rollback().await?; return Err(QualityError::NotFound("non-conformance")); }
            Some("closed") => { tx.rollback().await?; return Err(QualityError::InvalidState("non-conformance is closed")); }
            _ => {}
        }
        let id = Uuid::new_v4();
        sqlx::query(
            r#"INSERT INTO quality.quality_actions
                 (id, company_id, non_conformance_id, action_type, procedure_id, status, description, due_date)
               VALUES ($1,(SELECT company_id FROM quality.non_conformances WHERE id=$2),
                       $2,$3::quality_action_type,$4,'open'::quality_action_status,$5,$6)"#,
        )
        .bind(id).bind(a.non_conformance_id).bind(&a.action_type).bind(a.procedure_id)
        .bind(&a.description).bind(a.due_date)
        .execute(&mut *tx)
        .await?;
        sqlx::query(
            r#"UPDATE quality.non_conformances SET status='in_progress'::non_conformance_status
               WHERE id=$1 AND status='open'::non_conformance_status"#,
        )
        .bind(a.non_conformance_id)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(id)
    }

    /// Mark an action completed (terminal). Idempotent — a re-complete is a no-op.
    pub async fn complete_action(&self, action_id: Uuid, now: DateTime<Utc>) -> Result<(), QualityError> {
        let moved = sqlx::query(
            r#"UPDATE quality.quality_actions
               SET status='completed'::quality_action_status, completed_at=$2
               WHERE id=$1 AND status <> 'completed'::quality_action_status AND (metadata->>'deleted_at') IS NULL"#,
        )
        .bind(action_id).bind(now)
        .execute(&self.pool)
        .await?;
        if moved.rows_affected() == 0 {
            // Either already completed (idempotent no-op) or not found.
            let exists: Option<Uuid> = sqlx::query_scalar(
                "SELECT id FROM quality.quality_actions WHERE id=$1").bind(action_id).fetch_optional(&self.pool).await?;
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
        let row = sqlx::query(
            r#"SELECT company_id, status::text AS status FROM quality.non_conformances
               WHERE id=$1 AND (metadata->>'deleted_at') IS NULL FOR UPDATE"#,
        )
        .bind(nc_id)
        .fetch_optional(&mut *tx)
        .await?;
        let row = match row {
            Some(r) => r,
            None => { tx.rollback().await?; return Err(QualityError::NotFound("non-conformance")); }
        };
        let status: String = row.get("status");
        if status == "closed" {
            tx.rollback().await?;
            return Ok(()); // idempotent
        }
        let open_actions: i64 = sqlx::query_scalar(
            r#"SELECT count(*) FROM quality.quality_actions
               WHERE non_conformance_id=$1 AND status <> 'completed'::quality_action_status"#,
        )
        .bind(nc_id)
        .fetch_one(&mut *tx)
        .await?;
        if open_actions > 0 {
            tx.rollback().await?;
            return Err(QualityError::InvalidState("cannot close — incomplete actions remain"));
        }
        let company_id: Uuid = row.get("company_id");
        sqlx::query(
            r#"UPDATE quality.non_conformances SET status='closed'::non_conformance_status, closed_at=$2
               WHERE id=$1"#,
        )
        .bind(nc_id).bind(now)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        sink.publish(&QualityEvent::NonConformanceClosed(NonConformanceClosed { non_conformance_id: nc_id, company_id }));
        Ok(())
    }
}
