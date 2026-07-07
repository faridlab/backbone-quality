# backbone-quality — business flows & golden cases

## Flow: define → inspect → (reject) → NC → CAPA → close
```
create_template (parameters + acceptance criteria)
   │
   ▼  inspect → judge each reading vs its snapshotted criterion → ACCEPTED iff all pass
   │            → QualityInspectionCompleted{accepted, source} ── Stock subscribes (accept into stock / hold)
   │
   ├─ accepted → Stock accepts the goods
   └─ rejected → raise_non_conformance (source = the rejected inspection)
                   │
                   ▼  add_quality_action* (CAPA; NC → in_progress)
                   ▼  complete_action*
                   ▼  close_non_conformance (only when every action completed) → NonConformanceClosed
```
Quality posts NO GL and drives no neighbour — inbound it *reads* the triggering Stock receipt; outbound
it *signals* via events.

## Golden cases (`tests/quality_golden_cases.rs`)
- **QGC-1 — all in spec → accepted.** Diameter 10.0 ∈ [9.5,10.5] + Color manual-pass → accepted; event
  carries accepted=true.
- **QGC-2 — out of spec → rejected.** Diameter 11.2 > 10.5 → the inspection is rejected; the diameter
  reading is `rejected` while the in-spec color reading stays `accepted`.
- **QGC-3 — NC→CAPA→close.** A rejected inspection raises an NC (→ in_progress on first action); close is
  refused while the action is open; completing the action lets the NC close.
- **QGC-4 — validation.** Template needs a parameter; numeric min ≤ max; inspection needs a reading; a
  reading must name a template parameter.

## Integrity probes (`tests/integrity_probes.rs`)
- **IP-1 — NC requires a rejected inspection.** Citing an accepted inspection is refused.
- **IP-2 — close blocked by an incomplete action.** Two actions; the NC closes only once both complete.
- **IP-3 — no action on a closed NC.**
- **IP-4 — complete_action idempotent.** A retry is a no-op.
- **IP-5 — numeric reading needs a value.** A numeric parameter with no measured value → rejected.
- **IP-6 — verdict requires full parameter coverage (maturity).** A two-parameter template inspected with
  one reading is refused (not accepted); a duplicate reading is refused; full coverage passes — `accepted`
  means the whole template was measured, not just the readings sent.
- **IP-7 — inspection_type routable on the event (completeness).** An in-process (source-less) and an
  incoming inspection both emit a disposition event, and the events are distinguishable by
  `inspection_type` so a future WIP consumer can route the WIP verdict.

## Seam (`tests/quality_inspection_seam.rs`)
- **QSEAM-1 — inspect a REAL purchase receipt.** A real `inventory.purchase_receipts` row is created; the
  inspection links it (`source_type`/`source_id`) and the disposition event carries the source.

## §5 round-trip (`scripts/quality_inspection_seam_roundtrip.sh`)
Regen (`--force`) leaves the engine + event files byte-identical; the oracle + seam re-run green.
