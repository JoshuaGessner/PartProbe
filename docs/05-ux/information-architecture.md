# Information Architecture

## Metadata

- **Status:** Draft
- **Last updated:** 2026-07-29
- **Related requirement IDs:** UX-001 through UX-012, UX-021 through UX-045, REQ-F-001 through REQ-F-065
- **Related architecture decision IDs:** ADR-0001
- **Open questions:** Which roles can see pricing and rate-card administration in the first slice?
- **Dependencies:** [Screen inventory](screen-inventory.md), [workflow wireframes](workflow-wireframes.md)
- **Supersedes / superseded by:** None

## Structural model

The application has three levels: **Command center** (work to do), **records** (RFQs, quotes, customers, parts), and **workspaces** (the focused evidence and edits for one object). The global sidebar changes records; workspace tabs change the current object’s activity. This prevents a generic dashboard from competing with concentrated estimating work.

```text
Command center
├─ First-run/basic Rate Setup
├─ RFQ inbox
├─ Quotes
│  └─ Quote workspace: Overview | Parts | Routing | Analysis | Costs | Risks | Approval | Output
├─ Parts / model library
├─ Customers
├─ Actuals, CAM & corrections
├─ Capacity & availability (advisory)
├─ Libraries: Materials | Tools | Machines | Rates | Vendors | Templates
└─ Administration: Users | Backup & restore | Deployment settings

Part workspace (opened from Quote)
├─ Package & requirements
├─ Model review / geometry validation
├─ Features
├─ Stock
├─ Setups
├─ Requirements & PMI
├─ Revisions
└─ Analysis history
```

## Object relationships and navigation

- A quote owns revisions and quantity scenarios; a revision references immutable part-analysis snapshots.
- A part may appear on multiple quotes but its imported model revision/hash is distinct from the quote’s planning choices.
- Global search finds IDs, part numbers, customer, material, machine, and saved view—not hidden file contents by default.
- Breadcrumbs identify `Customer / RFQ / Quote revision / Part revision`; a pinned estimate strip shows cost, risk, price, and confidence in all quote workspace tabs.
- Deep links carry a record ID and tab, not an unvalidated local filesystem path.

## Primary task routes

| Intent | Route / workspace | Primary exit |
|---|---|---|
| Configure calculation inputs | Rate Setup | Confirm organization currency/rounding and resolve required rate categories without product-supplied values |
| Triage received request | RFQ inbox | Create/assign quote or decline with reason |
| Understand a model | Part workspace → Model review | Accept unit/validation snapshot or record uncertainty |
| Build a manufacturing proposal | Quote → Routing | Save proposed operations and setup alternatives |
| Compare advanced scenarios | Quote → Analysis | Explicitly adopt a route/view or retain baseline |
| Prove quote readiness | Part → Requirements & PMI | Resolve, exception, or block with evidence |
| Respond to revision | Part → Revisions | Branch a new draft and select explained deltas |
| Explain a price | Quote → Costs/Risks | Approval review with traceable breakdown |
| Send result | Quote → Output | Generated customer quote preview/export |
| Learn from outcome | Actuals & variance | Controlled recommendation, never an automatic rule change |

The Analysis tab uses progressive disclosure: baseline first; then route alternatives, availability, uncertainty, capacity/economics, sourcing, and bid priority as separately labeled lenses. Stale/missing badges and the pinned version manifest remain visible. No composite score is allowed to hide a blocker or its component factors.

An estimate encountering a missing, unapproved, expired, or conflicting rate links directly to the relevant Rate Setup row and returns to the same estimate context after correction. Basic setup is separate from later advanced multi-user library administration.

## Saved views

Users may save table column order, filters, sort, density, and selected side panes per named view. Views contain no calculation authority and are shareable only according to deployment authorization.
