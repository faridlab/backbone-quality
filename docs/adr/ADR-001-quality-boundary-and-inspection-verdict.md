# ADR-001 — Quality boundary, the inspection verdict, and the event-only seam

Status: accepted · 2026-07-07 · Tier 4c (Service Delivery pillar; posts no GL)

## Context
Quality is the most self-contained module in the pillar. ERPNext's `quality_inspection` physically lives
in the `stock` folder, but it is a distinct bounded context — so it moves to backbone-quality with a
*logical* link back to the Stock document that triggered it (brief §6.1). Its defining value is the
**verdict**: an honest accept/reject computed from readings against a spec, which Stock then acts on.

## Decision
1. **Quality Inspection lives here, linked to Stock by logical reference.** An inspection holds
   `source_type` / `source_id` (e.g. a purchase receipt) and the `item_id` — logical FKs, no DB
   constraint, no import of Stock.
2. **Acceptance criteria are snapshotted at inspect time, and coverage is enforced.** `inspect` loads the
   whole template's criteria in one snapshot and requires every parameter to be measured exactly once (no
   missing / duplicate / unknown reading); each reading copies its parameter's criterion (numeric
   [min,max] or a manual spec) onto the reading, so a later template edit never rewrites a completed
   inspection. The inspection is **ACCEPTED iff every reading is accepted** (a numeric reading with no
   measured value cannot be accepted). "Accepted" therefore means the item conforms to the WHOLE template,
   not just to the readings the caller chose to send (maturity council 2026-07-07).
3. **The coupling is event-out + read-in — quality drives no neighbour (brief §5.3).** The accept/reject
   verdict is published as `QualityInspectionCompleted`; Stock subscribes to it to accept goods into
   stock or hold them. Quality never calls Stock. This is the one Tier-4 module with no forward-drive
   port — its siblings (CRM→party/selling, project→billing, support→project) drive a real neighbour;
   quality only signals.
4. **CAPA is a bounded loop.** A rejected inspection can raise a NonConformance; actions attach to it and
   it closes only once every action is completed. The NC row is locked across add/close so the
   completeness check can't be raced by a concurrent action.
5. **Posts no GL.** Quality owns no money.

## Consequences
- Turn quality off and nothing downstream breaks — Stock simply loses the accept/hold signal and accepts
  everything. It is a pure overlay on the receiving/manufacturing flow.
- Proven end-to-end (`tests/quality_inspection_seam.rs` reads a REAL backbone-inventory receipt) and
  survives regen (§5).

## Parking lot (each with a gate)
- **In-process / outgoing inspection** — the verdict core is trigger-agnostic and the disposition event
  carries `inspection_type` so a WIP/shipping consumer can route it (completeness council 2026-07-07), but
  only the **incoming** path has a wired trigger (a Stock purchase receipt) + consumer today. The
  in-process/outgoing *capability* is deferred — gate: a manufacturing (WIP) / shipping module that owns
  the trigger + consumer. Incoming is the shipped, proven path.
- **`quality_actions.procedure_id` inserted unvalidated** — a CAPA can cite a nonexistent/foreign
  procedure; gate: validate the procedure belongs to the company (mirror `create_procedure`).
- **Verdict certified only the sent readings** — FIXED (maturity council 2026-07-07): `inspect` judged
  the caller's readings without checking the template was fully covered, so a partial inspection could
  pass unmeasured parameters (a false-pass releasing goods). `inspect` now enforces full parameter
  coverage from one template snapshot (IP-6, proven-by-revert).
- **`inspect` idempotency + transactional event publish** — a retry duplicates the inspection + event;
  the disposition event fires after commit on an in-process sink. Gate: an idempotency key + go-live
  outbox. A `UNIQUE(inspection_id, parameter_name)` / `UNIQUE(template_id, parameter_name)` would make
  coverage a DB backstop, not write-path-only.
- **Auto-hold of received stock on reject** — the disposition is an event today; a driven "hold the
  receipt" call into inventory is deferred until inventory owns a quarantine/hold verb (it currently has
  only create/submit). Gate: an inventory disposition API + the go-live event bus.
- **Quality goals / reviews / meetings / feedback** — deferred (governance ceremony, PRD non-goals).
- **Sampling-plan engine / control charts** — deferred (SAP-QM depth, not SMB KEEP).
