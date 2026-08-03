<!-- council: date=2026-08-02 repo-type=module unit=backbone-quality focus=maturity roster=chair,skeptic,steelman,yagni-business,ddd-bounded-context,contract-seat,domain-expert -->

# Council — module:backbone-quality — focus: maturity

> ## ⚠️ CORRECTION — verified 2026-08-03 (supersedes G1 and rec #2)
>
> **G1 is INVALID.** The "2 of 7 entities have no schema YAML" finding was a file-count artefact:
> it counted 5 `*.model.yaml` files against 7 DB tables and concluded two entities were orphaned from
> the SSoT. In reality the child entities are **nested models in their parent's file** —
> `QualityInspectionParameter` is the second model in `schema/models/quality_inspection_template.model.yaml`,
> and `QualityInspectionReading` (with its `ReadingResult` enum) is the second model in
> `schema/models/quality_inspection.model.yaml`. Both are fully specified, FK their parent, match
> their migrations column-for-column, and are covered by `index.model.yaml`'s `imports:` (the 5 files
> collectively define all 7 entities + enums). This is correct aggregate co-location (Template↔Parameter,
> Inspection↔Reading), not a gap. **Recommendation #2 (restore the G1 SSoT) is therefore VOID** —
> authoring separate YAMLs would have created *duplicate, conflicting* model definitions. Wherever
> G1 / "regen bomb" / rec #2 appears below, read it as superseded by this correction.
>
> **What survived verification:** #1 (fail-loud test gating) was the real must-fix and is **DONE**
> (commits `f29955b`, `b4d4e03` — CI gates the verdict/outbox/seam tests against Postgres; the logic
> harness skips cleanly without a DB; the HTTP suite no longer false-greens). The source-of-truth and
> verification scorecard rows below are revised accordingly. The lesson: a council finding asserted
> from a count, not a read, was falsified the moment the schema was opened — verify before acting.

## Best call
Stand up **fail-loud test gating**: a CI workflow (workspace or module root) that runs `cargo test` against a real Postgres, plus fixes to the two false-quiet harness paths — `tests/common/mod.rs:19-21` (`PgPool::connect(...).expect("connect")` → a `sqlx::test`/migration-runner spawn that fails loud) and `crud_test_base.rs:130-134` (unreachable-server → `TestResult::success("SKIPPED…")` → must become a hard failure).

This is the single move because it is the root enabler. It converts every correctness property of this module from "verified by reading the source text" to "verified by a running system," and — decisively for the maturity lens — it is the *prerequisite* for safely settling G1: you cannot faithfully regen the two orphan entities against restored YAML without a net that catches drift, and `metaphor schema validate` is not available on this build to settle it any other way. The SKEPTIC wins the adjudication: a module whose tests panic without a database and report success when the server is unreachable is not verified at any maturity level, and the IP6 false-pass bug (a parameter-coverage hole that would release unmeasured goods) was caught by reading source *after* release — direct proof the suite has never gated a release.

- Residual negative value: G1's regen bomb stays **armed but latent** for the hours-to-days between landing the net and landing the SSoT restoration. No regen is scheduled in that window, so realized risk is ~0; the standing risk is one `metaphor make entity` / schema rebuild silently drifting ~40 generated files plus the verdict engine's hand-written repo methods (`list_criteria`, `lock_status`, `lock_for_close`, `count_incomplete`, `complete`, `mark_in_progress`).
- Reversibility: high — a CI yaml plus two harness edits are trivially reverted.
- Evidence to flip: point me at an existing workspace-level workflow that already runs this module's `cargo test` against a real DB with a fail-loud harness, and the call flips to G1 (restore the two missing schema files).

## Disagreement map
1. **"Compiles green + tests exist = verified" vs "text-only, nothing verified."** Steelman/contract lean on the clean `cargo check` and the on-target test *text*; Skeptic (backed by orchestrator line-level confirmation of `expect("connect")` and the skip→success path) shows the default environment never runs them and they panic/skip. **Crux:** is maturity a property of the test source existing, or of the test running and gating? Chair sides with Skeptic — maturity is a property of the running system; the Steelman's strengths (real QMS invariants, hardened DB, correct dual-write) are genuine but unverified.
2. **G1 severity: "regen-time bomb, cannot defend" vs "deferrable standing hazard."** Steelman itself concedes G1 is load-bearing-bad; orchestrator notes no regen has fired yet and `metaphor schema validate` couldn't confirm whether it is already-broken-today vs future-bomb. **Crux:** latent-but-armed vs active. Both seats agree it must close; the only disagreement is *sequencing relative to the test net*, which the Best call resolves (net first, G1 immediately after).
3. **G2 severity: "exported trait with no impl = missing" vs "forward-declared contract, no consumer, defer."** Contract-seat flags `pub trait QualityQueryService` as a declared outward contract with no backing impl; orchestrator confirms no sibling module imports it. **Crux:** is an unimplemented exported trait a maturity gap or intentional forward-declaration? Chair sides with the orchestrator/contract-seat-defer reading — deferrable, but doc-mark it so it stops re-flagging.
4. **Empty `event_handlers` / `SubscriptionRegistry`: "dead code / missing consumer wiring" vs "publisher extension points."** **Crux:** extension-point vs dead-code. Resolved by the BC language — Quality is a publisher of events outward (Stock subscribes), not a self-subscriber; these are extension points, not gaps. Folding into parking lot, not a live tension.

## Recommendations (ranked by leverage)
| # | Move | Leverage | Residual negative | Reversibility | Evidence to flip |
|---|------|----------|-------------------|---------------|------------------|
| 1 | **Stand up fail-loud test gating** — CI runs `cargo test` against a real Postgres; fix `tests/common/mod.rs:19-21` panic and `crud_test_base.rs:130-134` skip→success. *(= Best call)* | Highest — converts all "verified-by-reading" into "verified-by-running"; enables safe G1 regen settlement. | G1 bomb armed-but-latent for the short window before #2 lands (~0 realized risk; standing risk = silent drift of ~40 files + verdict-engine repo methods). | High — CI yaml + two harness edits. | An existing workspace CI already running this module's `cargo test` against a real DB, fail-loud. |
| 2 | **Restore G1 SSoT** — author `quality_inspection_reading.model.yaml` + `quality_inspection_parameter.model.yaml`, add them to `index.model.yaml` `imports:`, then guarded-regen to diff generated output against checked-in code; reconcile any drift inside CUSTOM markers / `*_custom.rs`. Sequenced *after* #1. | Disarms the regen bomb; literally answers the user's "nothing missing" for schema entities. | Small — reverse-engineering YAML from current code risks encoding a current-but-wrong shape (e.g. mis-scoping a column that happens to work today). | High — YAML. | A guarded regen against the restored YAML producing byte-identical output proves the YAML is faithful. |
| 3 | **Codify the verdict-engine regression contract** as the named must-pass set inside #1's CI: IP6 full-parameter-coverage, NC→CAPA→close lifecycle, criterion snapshot, ACCEPTED-iff-all-readings-pass, NC-cites-REJECTED, NC-close-requires-zero-incomplete-under-lock. | Makes the CI guard the *actual QMS invariants*, not just "something ran" — prevents the next IP6-class false-pass. | Without #1, none of this runs; with #1, residual is the effort to enumerate, which is small (rules already encoded in `quality_write_service.rs`). | High — test additions. | Invariants already enumerated in a checked-in regression spec → just wire it. |
| 4 | **Doc-mark `QualityQueryService` as forward-declared** (one-line `///` on the trait: "forward-declared read API; no impl until a sibling module consumes it"). Closes G2 as non-blocking tidy. | Stops the trait re-flagging on every future review; cheap. | A future consumer hits an unimplemented trait — compile-time-discoverable, so low surprise. | High — a comment. | A sibling module importing `QualityQueryService` flips this to a must-impl (orchestrator confirms none today). |

## Maturity scorecard
| Seat | Axis | Score (1–5) | One sentence why |
|------|------|------------|------------------|
| ddd-bounded-context | bounded-context language + stable contracts | 3 | Outward BC edge is clean (Quality publishes events, doesn't self-subscribe; clean separation from Stock), but two entities' aggregate parentage is undecided in the SSoT and survives only in generated code. |
| contract-seat | explicit/minimal outward contract | 4 | Outward surface is explicit and genuinely minimal (events + `QualityWriteService` + `New*` DTOs `pub use`'d, host owns the auth-middleware type by design), docked one point for the unimplemented `QualityQueryService` trait that reads as a contract without a backing impl. |
| domain-expert | ubiquitous language + can represent every real state/rule | 4 | Language is faithful (NC/CAPA/inspection/template/parameter/reading/verdict/procedure) and the write engine can represent every QMS rule the module owns; held from 5 only because nothing running verifies the rules hold under drift. |
| (focus) | source-of-truth integrity | ~~2~~ **4** | **Corrected 2026-08-03:** all 7 entities + their enums ARE modeled in `schema/models/` — `QualityInspectionParameter` nests inside the template file and `QualityInspectionReading` inside the inspection file (correct aggregate co-location). The original "5 of 7 / orphans / regen bomb" finding was a file-count artefact and is withdrawn. SSoT is intact; migrations were generated from these models. |
| (focus) | verification / CI maturity | ~~1~~ **4** | **Updated 2026-08-03 after rec #1 landed:** CI (`.github/workflows/test.yml`) now runs the verdict/outbox/seam tests against Postgres on every push; the logic harness skips cleanly when `DATABASE_URL` is unset instead of panicking; the HTTP suite is `#[ignore]`d and fail-loud. Was 1 at council time (no CI, panic-on-no-DB, false-green). |

## Parking lot
- **G2 `QualityQueryService` impl** — defer until a sibling consumes it; orchestrator-confirmed no consumer today (recommendation #4 only doc-marks it).
- **Aggregate boundary decision for `quality_inspection_reading` / `quality_inspection_parameter`** (ddd-seat) — decide whether they are independent aggregates or children of inspection/template; a design question, not a completeness blocker, but it shapes how #2's YAML is authored.
- **"Rejected inspection should/shouldn't auto-raise an NC"** (domain-seat) — citation-based design choice, likely intentional; confirm with a domain owner, not a maturity finding.
- **Consumer-side wiring / `SubscriptionRegistry` population** — extension points for when Quality needs to *react to* sibling events; YAGNI today (Quality is a publisher).
- **OpenAPI / gRPC feature gates, `versioning/version_compat.rs`** — touched in the working tree but out of every seat's finding and out of the maturity-completeness lens; revisit if a future focus is API-surface maturity.
