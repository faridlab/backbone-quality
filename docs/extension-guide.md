# backbone-quality — Extension Guide

## Public surface (stable)
- **Events** (`application::service::quality_events`): `QualityInspectionCompleted` (carries `accepted` +
  `inspection_type` + `source_type`/`source_id` — the disposition, routable by trigger: incoming → Stock
  accept/hold on `source_id`, in_process/outgoing → a future WIP/shipping consumer), `NonConformanceRaised`,
  `NonConformanceClosed`, the `QualityEvent` union, and `QualityEventSink` (a consuming service supplies
  its own sink — bus, outbox, …).
- **Write path** (`application::service::quality_write_service::QualityWriteService`): the guarded verbs
  (`create_template`, `create_procedure`, `inspect`, `raise_non_conformance`, `add_quality_action`,
  `complete_action`, `close_non_conformance`). Timestamped verbs take an explicit `now`.

## How a consuming service uses the disposition
Subscribe to `QualityInspectionCompleted`: on `accepted`, accept the referenced Stock document
(`source_type`/`source_id`) into stock; on `!accepted`, hold it and (optionally) call
`raise_non_conformance`. **Quality never calls Stock** — the flow is event-driven (brief §5.3), so the
subscriber owns the disposition action.

## Not a contract
- The 12 generated CRUD endpoints per entity (`BackboneCrudHandler`) are convenience scaffolding. Do
  **not** compute a verdict or close an NC through the generic PATCH surface — it bypasses the
  readings-vs-criteria judgment, the criterion snapshot, and the close-requires-completed-actions gate.
  Use `QualityWriteService`.
- `// <<< CUSTOM` blocks inside generated files preserve local edits only; not a cross-module extension
  point.

## Invariants a consumer must not break
- Acceptance criteria are snapshotted at inspect time; do not recompute a reading's verdict from the live
  template.
- An inspection is accepted only if EVERY reading passed; don't infer accept from a subset of readings.
- A non-conformance closes only when all its actions are completed.
