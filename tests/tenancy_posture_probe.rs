//! Tenancy posture probe (ADR-0029).
//!
//! The module ships NO tenancy: no tenant column, no tenant predicate in any statement,
//! and no RLS policy of its own. What it ships instead is the half-fence the composing
//! service's tenancy decorator completes: every table carries ENABLE + FORCE ROW LEVEL
//! SECURITY with zero policies. This probe pins that posture from below, the family
//! pattern (proven on backbone-accounting, then backbone-billing):
//!
//! - the flags are armed and the policy set is empty (schema pin);
//! - a plain non-superuser, NOBYPASSRLS role is default-DENIED — zero rows, writes
//!   refused — no matter what legacy variable is set (no policy reads `app.company_id`
//!   anymore; the decorator's org-scoped policies will, once composed);
//! - the owner/superuser pool sees its own seeded rows plainly, and the CAPA write path
//!   runs under the AMBIENT org scope — the per-request binding a composing service
//!   resolves — with the outbound events' legacy company twin echoing that scope; the
//!   same binding on a RESTRICTED pool returns exactly what the (absent) policies admit:
//!   nothing, until the decorator composes.
//!
//! Requires DATABASE_URL (:5433/backbone_quality) reachable as a superuser (to mint
//! and tear down the probe role).

use sqlx::{PgPool, Row};
use uuid::Uuid;

use backbone_quality::application::service::quality_events::{QualityEvent, QualityEventSink};
use backbone_quality::application::service::quality_write_service::{
    NewNonConformance, NewProcedure, QualityError, QualityWriteService,
};

const ROLE: &str = "quality_tenancy_probe";
const PWD: &str = "probe";

/// Role/catalog DDL serializes — two tests minting roles concurrently hit
/// "tuple concurrently updated" in the system catalogs.
static ROLE_DDL_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

async fn admin() -> PgPool {
    let url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://postgres:postgres@localhost:5433/backbone_quality".to_string()
    });
    PgPool::connect(&url).await.expect("connect admin")
}

/// Shed the role's grants, then drop it. Leftover grants (from a run whose teardown never
/// reached the drop, or whose drop was swallowed) make plain DROP ROLE fail with 2BP01 —
/// DROP OWNED BY first keeps both bootstrap and teardown idempotent across runs.
async fn drop_role(admin: &PgPool) {
    let _ = sqlx::query(&format!("DROP OWNED BY {ROLE}"))
        .execute(admin)
        .await;
    let _ = sqlx::query(&format!("DROP ROLE IF EXISTS {ROLE}"))
        .execute(admin)
        .await;
}

async fn bootstrap_role(admin: &PgPool, tables: &[&str]) {
    drop_role(admin).await;
    for stmt in [
        format!("CREATE ROLE {ROLE} LOGIN PASSWORD '{PWD}' NOSUPERUSER NOBYPASSRLS"),
        format!("GRANT USAGE ON SCHEMA quality TO {ROLE}"),
    ]
    .into_iter()
    .chain(tables.iter().map(|t| {
        format!("GRANT SELECT, INSERT, UPDATE ON TABLE quality.{t} TO {ROLE}")
    })) {
        sqlx::query(&stmt).execute(admin).await.unwrap();
    }
}

/// A local capturing sink (the shared `common` module belongs to the other suites;
/// keeping this one local keeps the probe's dependency surface explicit).
#[derive(Default, Clone)]
struct CapturingSink {
    events: std::sync::Arc<std::sync::Mutex<Vec<QualityEvent>>>,
}

impl CapturingSink {
    fn events(&self) -> Vec<QualityEvent> {
        self.events.lock().unwrap().clone()
    }
}

impl QualityEventSink for CapturingSink {
    fn publish(&self, event: &QualityEvent) {
        self.events.lock().unwrap().push(event.clone());
    }
}

/// Raise one non-conformance and return the legacy company twin its event carried.
async fn raised_event_company(w: &QualityWriteService, subject: &str) -> Uuid {
    let sink = CapturingSink::default();
    w.raise_non_conformance(
        NewNonConformance {
            subject: subject.into(),
            source_inspection_id: None,
            item_id: None,
            severity: "low".into(),
            description: None,
        },
        chrono::Utc::now(),
        &sink,
    )
    .await
    .expect("NC raise completes");
    sink.events()
        .into_iter()
        .find_map(|e| match e {
            QualityEvent::NonConformanceRaised(r) => Some(r.company_id),
            _ => None,
        })
        .expect("the raise published NonConformanceRaised")
}

// ── The schema pin: armed flags, empty policy set ─────────────────────────────

/// Every quality base table carries ENABLE + FORCE ROW LEVEL SECURITY and the
/// module ships ZERO policies — the decorator's half-fence. If a strip or regen ever
/// drops the flags, an undecorated deployment would silently become readable by any
/// role the host grants; if a policy ever reappears module-side, the decorator's
/// org-scoped policies would fight it. (The outbox's company artifacts — index and
/// policy — were stripped with the rest; its `company_id` COLUMN stays because the
/// framework's `stage()` inserts it NOT NULL.)
#[tokio::test]
async fn tables_carry_rls_flags_and_the_module_ships_no_policy() {
    let admin = admin().await;
    let armed: Vec<String> = sqlx::query(
        "SELECT c.relname FROM pg_class c \
         JOIN pg_namespace n ON n.oid = c.relnamespace \
         WHERE n.nspname = 'quality' AND c.relkind = 'r' \
           AND c.relrowsecurity AND c.relforcerowsecurity \
         ORDER BY c.relname",
    )
    .fetch_all(&admin)
    .await
    .unwrap()
    .iter()
    .map(|r| r.get::<String, _>("relname"))
    .collect();
    for table in [
        "non_conformances",
        "outbox_events",
        "quality_actions",
        "quality_inspection_parameters",
        "quality_inspection_readings",
        "quality_inspection_templates",
        "quality_inspections",
        "quality_procedures",
    ] {
        assert!(
            armed.iter().any(|t| t == table),
            "{table} must carry ENABLE + FORCE ROW LEVEL SECURITY"
        );
    }

    let policies: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM pg_policy WHERE polrelid::regnamespace::text = 'quality'",
    )
    .fetch_one(&admin)
    .await
    .unwrap();
    assert_eq!(
        policies, 0,
        "the module ships no RLS policy — isolation belongs to the composing service's decorator"
    );
}

// ── Default-deny until composed: the plain probe role ─────────────────────────

/// A plain non-superuser, NOBYPASSRLS role with bare grants sees NOTHING and cannot
/// write — with or without the legacy company variable set. No policy admits it (there
/// are none), and none reads `app.company_id` anymore. The owner pool still sees its
/// seeded row: the denial is the missing policy, not an empty database.
#[tokio::test]
async fn plain_role_is_default_denied_until_the_decorator_composes() {
    let _ddl = ROLE_DDL_LOCK.lock().await;
    let admin = admin().await;
    bootstrap_role(&admin, &["quality_procedures", "non_conformances"]).await;

    // The owner seeds a procedure as the superuser (whom RLS can never bind). No tenant
    // column exists to set — a procedure is just a row (ADR-0029).
    let procedure = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO quality.quality_procedures
             (id, procedure_name, status)
           VALUES ($1, 'tenancy probe', 'active'::quality_procedure_status)"#,
    )
    .bind(procedure)
    .execute(&admin)
    .await
    .unwrap();

    let restricted = PgPool::connect(&format!(
        "postgresql://{ROLE}:{PWD}@localhost:5433/backbone_quality"
    ))
    .await
    .expect("connect probe role");

    // Bare read: zero rows — default-deny with no policy admitting the role.
    let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM quality.quality_procedures WHERE id=$1")
        .bind(procedure)
        .fetch_one(&restricted)
        .await
        .unwrap();
    assert_eq!(n, 0, "a role no policy admits sees zero rows");

    // The legacy company variable resurrects nothing: no policy reads it anymore
    // (the decorator's org-scoped policies will, once composed).
    let mut tx = restricted.begin().await.unwrap();
    sqlx::query("SELECT set_config('app.company_id', $1, true)")
        .bind(Uuid::new_v4().to_string())
        .execute(&mut *tx)
        .await
        .unwrap();
    let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM quality.quality_procedures WHERE id=$1")
        .bind(procedure)
        .fetch_one(&mut *tx)
        .await
        .unwrap();
    assert_eq!(n, 0, "the legacy variable must not bypass the absent policy set");
    tx.rollback().await.unwrap();

    // A write is refused outright (no WITH CHECK policy admits the new row).
    let err = sqlx::query(
        r#"INSERT INTO quality.quality_procedures
             (id, procedure_name, status)
           VALUES ($1, 'probe write', 'active'::quality_procedure_status)"#,
    )
    .bind(Uuid::new_v4())
    .execute(&restricted)
    .await;
    assert!(err.is_err(), "a write with no admitting policy must be refused");

    // The owner pool still sees its row.
    let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM quality.quality_procedures WHERE id=$1")
        .bind(procedure)
        .fetch_one(&admin)
        .await
        .unwrap();
    assert_eq!(n, 1, "the owner pool must still see the seeded row");

    drop_role(&admin).await;
}

// ── The module-side half: the ambient org scope drives the module ─────────────

/// Run `f` with an ambient org scope bound — the single-company emulation of what a
/// composing service resolves and binds per request.
async fn scoped<F, R>(pool: &PgPool, company: Uuid, f: F) -> R
where
    F: std::future::Future<Output = R>,
{
    backbone_orm::org_scope::with_org_request_scope(
        pool,
        backbone_orm::org_scope::OrgScope::for_company_unit(company),
        f,
    )
    .await
    .unwrap()
}

/// The ambient org scope is what the module's write path rides: with a scope bound
/// (the composed shape), the CAPA flow completes and the outbound event's legacy
/// company twin echoes the bound scope; without one, nothing is bound and the echo
/// is nil — an uncomposed deployment never forges a tenant on the wire. Row isolation
/// itself is the decorator's — this pins the module-side binding contract only.
#[tokio::test]
async fn ambient_org_scope_drives_module_writes_and_the_legacy_twin_echo() {
    let admin = admin().await;
    let company = Uuid::new_v4();
    let w = QualityWriteService::new(admin.clone());

    // Inside the scope: bound, visible to module code, the write completes, and the
    // raised-NC event echoes the scope's legacy company id.
    let (legacy_inside, procedure, echo_inside) = scoped(&admin, company, async {
        let scope = backbone_orm::org_scope::current_org_scope()
            .expect("the ambient scope must be bound inside");
        let procedure = w
            .create_procedure(NewProcedure {
                procedure_name: "scoped probe".into(),
                parent_procedure_id: None,
                description: None,
            })
            .await
            .expect("procedure write completes under the ambient scope");
        let echo = raised_event_company(&w, "scoped probe nc").await;
        (scope.legacy_company_id(), procedure, echo)
    })
    .await;
    assert_eq!(legacy_inside, Some(company));
    assert_ne!(
        procedure, Uuid::nil(),
        "the procedure write completed under the ambient scope"
    );
    assert_eq!(
        echo_inside, company,
        "the outbound event's legacy twin echoes the bound scope"
    );

    // Outside any scope the twin falls back to nil.
    let echo_outside = raised_event_company(&w, "unscoped probe nc").await;
    assert_eq!(
        echo_outside,
        Uuid::nil(),
        "unscoped raises must not forge a tenant on the outbound wire"
    );

    // Nothing leaks past the wrapped future.
    assert!(
        backbone_orm::org_scope::current_org_scope().is_none(),
        "no ambient scope may leak past the wrapped future"
    );
}

/// The relay shape a decorated host runs: the app role (restricted, NOBYPASSRLS) with
/// the ambient scope bound per request. The binding rides the request-dedicated
/// connection, a module read completes, and it returns exactly what the (still absent)
/// policies admit: nothing — the owner's procedure reads as absent, so citing it
/// refuses the write, indistinguishable from a missing row.
#[tokio::test]
async fn restricted_pool_with_ambient_scope_completes_default_denied() {
    let _ddl = ROLE_DDL_LOCK.lock().await;
    let admin = admin().await;
    bootstrap_role(&admin, &["quality_procedures", "non_conformances"]).await;
    let restricted = PgPool::connect(&format!(
        "postgresql://{ROLE}:{PWD}@localhost:5433/backbone_quality"
    ))
    .await
    .expect("connect probe role");

    // The owner seeds a procedure the probe role must NOT see.
    let procedure = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO quality.quality_procedures
             (id, procedure_name, status)
           VALUES ($1, 'owner only', 'active'::quality_procedure_status)"#,
    )
    .bind(procedure)
    .execute(&admin)
    .await
    .unwrap();

    let company = Uuid::new_v4();
    let w = QualityWriteService::new(restricted.clone());
    let refused = scoped(&restricted, company, async {
        w.create_procedure(NewProcedure {
            procedure_name: "cross-tenant probe".into(),
            parent_procedure_id: Some(procedure),
            description: None,
        })
        .await
    })
    .await
    .expect_err("the foreign parent reads as absent → the write is refused");
    assert!(
        matches!(refused, QualityError::Invalid(_)),
        "a mismatched tenant's procedure is indistinguishable from a missing one"
    );
    assert!(
        backbone_orm::org_scope::current_org_scope().is_none(),
        "no ambient scope may leak past the wrapped future"
    );

    drop_role(&admin).await;
}
