# backbone-quality — PRD

Tier 4c · Service Delivery pillar · posts **no GL** · drives no neighbour (event-coupled).

## Why
An Indonesia SMB that receives goods or manufactures needs to check quality against a spec and act on
failures — without the depth of SAP QM. This is the lean quality core: define what "good" means (a
template of parameters + acceptance criteria), inspect an item against it (readings → accept/reject),
and when something fails, record a non-conformance and drive it to closure with corrective/preventive
actions. The value is the **verdict** — an honest accept/reject that Stock can act on.

## Scope (KEEP — pillar brief §4/§6/§7)
- **QualityInspectionTemplate (+ QualityInspectionParameter)** — the acceptance-criteria master: per
  parameter, a numeric [min,max] range or a free-text spec.
- **QualityInspection (+ QualityInspectionReading)** — an inspection against a template. Each reading is
  judged against its parameter's **snapshotted** criterion; the inspection is **accepted iff every
  reading passes**, else **rejected**. It links the Stock document that triggered it (`source_type` /
  `source_id`, logical FK) and the item.
- **NonConformance** — a recorded failure (from a rejected inspection or an audit), gathering CAPA.
- **QualityAction (CAPA)** — corrective/preventive actions; an NC closes only once its actions complete.
- **QualityProcedure** — a lightweight adjacency-list tree of documented procedures a CAPA enacts.
- **Coupling is event-out + read-in** (brief §5.3) — the accept/reject verdict is published as
  `QualityInspectionCompleted` (the signal Stock subscribes to for "accept into stock / hold"); the
  inspection *reads* a real upstream Stock receipt. Quality **drives no neighbour** and posts no GL.

## Non-goals (CUT / DEFER — brief §4)
- **In-process / outgoing inspection triggers** — the verdict core is trigger-agnostic and the disposition
  event routes by `inspection_type`, but only the **incoming** path (a Stock purchase receipt trigger + a
  Stock consumer) is wired today. In-process (WIP) and outgoing (pre-delivery) triggers/consumers are
  DEFERRED until a manufacturing / shipping module owns that seam (ADR-001).
- **Quality goals / reviews / meetings / feedback** — governance ceremony, low SMB ROI (DEFER).
- Deep SAP-QM inspection plans, sampling-plan engines, control charts.
- Any GL posting; any driven write into Stock (the disposition is an event Stock consumes).

## Success criteria
- The verdict is exact: readings vs criteria → accept/reject, criteria snapshotted so a later template
  edit never rewrites a completed inspection (golden cases).
- The CAPA loop closes correctly — an NC can't close with an incomplete action.
- An inspection correctly links a REAL upstream Purchase Receipt and emits the disposition (proven
  against REAL backbone-inventory).
- Zero normal Cargo edge; survives a full codegen regen (§5).
