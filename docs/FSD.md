# backbone-quality — FSD

## Entities
QualityInspectionTemplate (`item_id`, `is_active`) · QualityInspectionParameter (`numeric`, `min_value`,
`max_value`, `spec_text`) · QualityInspection (`template_id`/`item_id`/`source_type`/`source_id` logical
FKs, `inspection_type`, `sample_size`, `status`) · QualityInspectionReading (`reading_value`, snapshotted
`min_value`/`max_value`, `manual_result`, `result`) · NonConformance (`source_inspection_id`, `severity`,
`status`) · QualityAction (`non_conformance_id`, `action_type`, `procedure_id`, `status`) ·
QualityProcedure (`parent_procedure_id` tree). Enums: InspectionType {incoming, in_process, outgoing},
InspectionStatus {pending, accepted, rejected}, ReadingResult {accepted, rejected}, NonConformanceSeverity
{low, medium, high, critical}, NonConformanceStatus {open, in_progress, closed}, QualityActionType
{corrective, preventive}, QualityActionStatus {open, in_progress, completed}.

## Write path (`QualityWriteService`, hand-authored, user-owned)
- `create_template` / `create_procedure`
- `inspect(NewInspection, now, sink)` → `InspectOutcome{inspection_id, accepted}` — the verdict engine
- `raise_non_conformance(now, sink)` / `add_quality_action` / `complete_action(now)` /
  `close_non_conformance(now, sink)` — the CAPA loop (NC + close row-locked)

Timestamps passed in (`now`) for deterministic tests. Errors: `QualityError {Db, NotFound, InvalidState,
Invalid}`.

## Seam (event-out + read-in — zero normal Cargo edge)
- **Inspect a real receipt (proven, QSEAM-1):** an incoming inspection links a REAL
  backbone-inventory Purchase Receipt (`source_type='purchase_receipt'`, `source_id=<real receipt>`) and
  emits `QualityInspectionCompleted` carrying the source, so Stock can accept/hold the right document.
- **Outbound:** the accept/reject verdict is an **event** Stock subscribes to (brief §5.3) — quality
  drives no neighbour and posts no GL. This is the one Tier-4 module with no forward-drive port.

## Test oracle
`quality_golden_cases` (4: all-in-spec accepted, out-of-spec rejected, NC→CAPA→close flow, validation),
`integrity_probes` (7: NC-requires-rejected-inspection, close-blocked-by-incomplete-action,
no-action-on-closed-NC, complete-action-idempotent, numeric-reading-needs-value, IP-6 verdict-requires-
full-parameter-coverage, IP-7 inspection-type routable on the disposition event), `quality_inspection_seam`
(1: inspect a REAL inventory purchase receipt) + §5 round-trip. **12 tests.**

> The generated `integration_tests.rs` hits an external HTTP server (`API_BASE_URL`, default
> `127.0.0.1:3000`) and is environmental scaffolding, not part of this module's correctness gate; the
> hand-authored oracle above + §5 is the gate.
