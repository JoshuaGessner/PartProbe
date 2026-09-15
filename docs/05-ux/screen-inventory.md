# Screen Inventory

## Metadata

- **Status:** Draft
- **Last updated:** 2026-09-14
- **Related requirement IDs:** UX-001 through UX-045, REQ-F-001 through REQ-F-065
- **Related architecture decision IDs:** ADR-0001
- **Open questions:** Prioritize library/admin screens after initial vertical-slice validation.
- **Dependencies:** [Information architecture](information-architecture.md), [workflow wireframes](workflow-wireframes.md)
- **Supersedes / superseded by:** None

## Inventory

| Screen / workspace | Purpose | Slice | Key evidence / action |
|---|---|---|---|
| Command center | Recent work, assigned RFQs, analysis/approval exceptions | 1 | Open/create/continue safely |
| New estimate | Upload-first path from local model through vital inputs, reviewed proposals, and transparent result | 1 | Model/analysis state, Settings readiness, minimal unresolved facts, cost/price trace |
| RFQ inbox | Triage and assign incoming packages | 1 | Status, due date, package completeness |
| Quote list | Find/filter quotes and revisions | 1 | Owner, customer, value, confidence, status |
| Quote workspace overview | Quote context and persistent total strip | 1 | Assumptions, next review action |
| Part/package | Link model, drawing, specs, revision metadata | 1 | Hash, attachments, requirement gaps |
| Model import | Controlled intake and file validation | 1 | Format, local handling, unit candidate |
| Model and stock review | View model, proposed stock, integrity, and basic properties without giving the renderer calculation authority | 1 / VIS-1 phases A/B, VIS-2 source display, and initial VIS-3 proposed-stock review implemented as an internal preview; later VIS-3/VIS-4/VIS-5 work remains | Contract v12 exposes a clickable same-window destination, native viewport, return navigation, four semantic keyboard-operable standard views, and a path-free text equivalent for representation, units, dimensions, warnings, confidence, analysis reference, and available proposal facts only behind the deliberate internal build profile. Primary navigation reports the current page, and the refreshed macOS package proves a successful pointer or keyboard switch focuses the exact new workspace heading with a visible keyboard focus ring. VoiceOver/Narrator/Orca comprehension remains pending. The selected exact STEP's validated display derivative reaches the native renderer; geometry does not reach the WebView. A matching proposal adds translucent stock under `proposed-stock-display-placement-v1`. Preferred-adapter failure requests one fallback adapter; startup or unrecovered surface/device failure expands the same screen into text-only review. Editable stock candidates, standard size/availability, machine-fit authority, forced fallback/device-loss platform evidence, complete screen-reader evidence, and three-OS package acceptance remain |
| Feature review | Inspect/add/accept/reject features | 3 | Geometry link, confidence, impact |
| Stock selection | Compare editable stock candidates | 1 / VIS-4 planned | Numeric envelope/allowance edits create immutable reviewed candidates; no drag-only or silent centered-placement assumption |
| Setup/orientation review | Compare setup alternatives | 2 | Access/workholding confidence |
| Routing editor | Build ordered operations | 1 | Times, machine, source, overrides |
| Tool and feeds/speeds review | Validate proposed tooling parameters | 3 | Versioned source and constraints |
| Material selector | Select priced/qualified material | 1 | Spec, form, availability, price freshness |
| Workcenter selector | Match operation and capability | 1 | Capability separate from financial rate |
| Operation editor | Detailed operation and time/cost inputs | 1 | Formula/source/override history |
| Cost breakdown | Explain internal cost and price | 1 | Cost vs risk vs price, drill-down |
| Quantity comparison | Compare breaks/scenarios | 1 | Make qty, amortization, margin guard |
| Risk review | Identify/resolve uncertainty and allowance | 1 | Owner, impact, mitigation, acceptance |
| Approval review | Internal approval record | 1 | Thresholds, snapshot, exceptions |
| Customer quote preview | Controlled outward-facing output | 1 | Exclusions, revision, export/print |
| Shop Settings | Configure currency, rates, material/stock prices and allowances, machines/runtime profiles, and pricing policy without product numeric defaults | 1 | Current GUI separates Rates & pricing from Resources, persists rate/pricing plus one optional starter bundle, and can load the explicit unconfirmed `huntsville-2026q3-test-*` usability profile. Schema-v3 catalogs remain read-only/identity-gated. Validated multi-entry editing, dirty-state/discard handling, authenticated identity/roles, a reviewed allow configuration, and activation remain pending |
| Advanced material/tool/machine/rate/vendor libraries | Govern reusable multi-user shop data | 4 | Version, approval, import staging, history |
| Customer management | CRM and pricing policy context | 2 | Contacts, restrictions, history |
| Historical actuals / variance | Compare original estimate and outcome | 6 | Bias evidence; controlled recommendation |
| Route comparison | Compare feasible alternatives without replacing baseline | 3 | exclusion reasons, cost/time/setup/risk, adoption record |
| Capacity and economics | Advisory delivery, occupancy, contribution and opportunity views | 7 | snapshot/freshness, bottleneck, separate cost bases |
| Uncertainty and sensitivity | Explain ranges and material drivers | 7 | distribution assumptions, percentiles, seed/version |
| Requirement coverage | Confirm interpretation/applicability/coverage/verification | 5 | evidence, owner, blocker/exception history |
| Revision comparison | Review geometry-to-cost differences across revisions | 3 | mapping confidence, unknowns, estimate/rate basis |
| Feature cost/risk view | Drill from cost categories to features/requirements/operations | 3 | allocation method and unallocated remainder |
| Availability/readiness | Apply timestamped tool/fixture/material/workcenter state | 2 | theoretical vs current, unknown/stale state |
| Make/buy comparison | Compare eligible internal and external alternatives | 2 | landed cost, constraints, quote freshness |
| CAM reconciliation | Compare estimate, CAM, machine observation, and actual | 6 | typed source, mapping, cause and variance |
| Correction/rule review | Govern correction cohorts and rule suggestions | 6 | bias/sample/privacy/backtest/activation |
| Bid and quote priority | Show blockers and explainable score components | 2 | policy version, evidence, human decision |
| Templates | Reusable routing/quote structures | 4 | Scope and provenance |
| Users / backup / restore | Administration | 8 | Authorization and recovery evidence |

Slice numbering maps to the roadmap phases. Phase 2 intentionally includes only one route, a basic requirement checklist, manual availability/readiness, and manual make/buy comparison; advanced engines arrive in later phases. Screens not in the current slice may have read-only placeholder links only when they help navigation; do not build empty shells to simulate scope.
