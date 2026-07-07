# backbone-quality — BRD

## Documents
QualityInspectionTemplate (+ QualityInspectionParameter) · QualityInspection (+
QualityInspectionReading) · NonConformance · QualityAction (CAPA) · QualityProcedure. Own Postgres
schema `quality`. Posts NO GL, drives no neighbour.

## Business rules

**BR-1 (define a template).** `create_template` requires a name + ≥ 1 parameter. A numeric parameter
needs a coherent range (min ≤ max when both given, and at least one bound); a non-numeric parameter
carries a free-text spec judged by a manual pass/fail.

**BR-2 (inspect — the verdict).** `inspect` loads the whole template's criteria in one snapshot and
requires **full coverage**: every template parameter must have exactly one reading — a missing,
duplicate, or unknown reading is refused (an `accepted` verdict must mean the item conforms to the
TEMPLATE, not just to the readings the caller chose to send — maturity council 2026-07-07). Each reading
is judged against its parameter's criterion: numeric → the measured value within [min,max] (a numeric
reading with **no value** cannot be accepted); non-numeric → a manual pass. The criterion is
**snapshotted** onto the reading. The inspection is **ACCEPTED iff every reading is accepted**, else
**REJECTED**. Emits `QualityInspectionCompleted{accepted, source_type, source_id}` — the disposition
Stock subscribes to.

**BR-3 (raise a non-conformance).** `raise_non_conformance` needs a subject; if it cites a source
inspection, that inspection must be **rejected** (you don't raise an NC on a passing inspection). Status
`open`. Emits `NonConformanceRaised`.

**BR-4 (CAPA).** `add_quality_action` attaches a corrective/preventive action to an **open/in_progress**
NC and advances it to `in_progress`; the NC row is locked so a concurrent close can't slip a fresh
action past it. `complete_action` marks an action completed (idempotent). No action may be added to a
**closed** NC.

**BR-5 (close a non-conformance).** `close_non_conformance` closes an NC **only once every action is
completed** — an incomplete action blocks the close (row-locked, so it serializes against a concurrent
`add_quality_action`). Idempotent. Emits `NonConformanceClosed`.

**BR-6 (procedure tree).** `create_procedure` adds a procedure to a lightweight adjacency-list tree
(`parent_procedure_id`, same company).

## Events
`QualityInspectionCompleted`, `NonConformanceRaised`, `NonConformanceClosed`.

## Deferred (with reason)
Quality goals / reviews / meetings / feedback (governance ceremony — low SMB ROI), sampling-plan
engines, control charts, any driven write into Stock (the disposition is an event Stock consumes).
