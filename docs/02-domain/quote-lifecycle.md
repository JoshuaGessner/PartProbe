# Quote Lifecycle

> **Status:** Draft
> **Last updated:** 2026-07-29
> **Related requirements:** REQ-F-001, REQ-F-009–REQ-F-012, REQ-F-049–REQ-F-053, REQ-F-060–REQ-F-065
> **Related ADRs:** ADR-0006, ADR-0008–ADR-0014
> **Open questions:** OQ-023–OQ-026, OQ-038–OQ-050
> **Dependencies:** Approval and permissions policy
> **Supersedes:** None

`Received → Triaged → Estimating → Technical review → Commercial review → Approved → Issued → Won/Lost/Expired/Withdrawn`.

Bid/no-bid, quote priority, recommendation, blockers, approvals, overrides, and decline reasons are first-class versioned decisions. A score informs but never performs management approval.

Each issued quote references exactly one approved estimate revision, adopted routing alternative, requirement-coverage snapshot, source/model revision, selected rate-entry/card and selector versions, pricing policy, rounding policy, calculation-rule versions, and rendering template version. Capacity, availability, uncertainty, opportunity-cost, sourcing, and scoring snapshots used for approval are pinned when applicable but remain internal unless an approved template says otherwise.

A customer/model/PMI/requirement change creates a new branch with a visible geometry/feature/requirement/routing/cost diff; it does not mutate issued evidence. Quote readiness blocks unresolved critical requirements unless authorized exception evidence is part of the new revision. Conversion to production or CAM exports reviewed planning data but does not represent a production router or CAM program.
