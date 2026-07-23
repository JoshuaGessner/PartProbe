# Requirement Coverage Workflow

> **Status:** Draft  
> **Last updated:** 2026-07-22  
> **Related requirements:** REQ-F-051–REQ-F-054, REQ-F-065; REQ-NF-015–REQ-NF-018, REQ-NF-020–REQ-NF-022  
> **Related ADRs:** ADR-0001, ADR-0004, ADR-0008, ADR-0012  
> **Open questions:** Default blocker policy; exception roles; source-precedence presentation; required customer-facing exception language  
> **Dependencies:** Requirement-coverage model, quote lifecycle, permissions matrix, PMI importer, design system/accessibility  
> **Supersedes:** None

## Goal

Help an estimator prove what requirements were found and addressed without implying that automation guarantees completeness. The workflow keeps source evidence, interpretation, applicability, manufacturing/inspection/document/security impact, cost linkage, and quote readiness visible as distinct decisions.

## Entry and summary

The quote workspace shows a persistent coverage summary: `sources reviewed`, `requirements`, `recognized and costed`, `needs clarification`, `conflicts`, `missing source`, and `readiness blockers`. Counts are links, use text/icons as well as color, and state the evaluated package/revision and policy version. `No detected requirements` is never displayed as `No requirements`.

Readiness has four labels: `Not ready`, `Ready with warnings`, `Ready with exceptions`, and `Ready`. Only the versioned evaluator produces this state. A banner lists the highest-severity blockers and offers `Review blockers`; it never offers a one-click “mark all complete.”

## Workspace

- **Left — sources:** model/PMI, drawings, notes, specifications, RFQ/PO clauses, customer rules and user entries with revision, classification and review/completeness state.
- **Center — coverage matrix:** sortable/filterable requirement rows with category, applicability, interpretation, coverage, verification, owner, cost/operation links, conflicts and blocker state.
- **Right — evidence and impact inspector:** exact source locator/preview, normalized value, importer/extractor confidence, geometry highlight when available, linked routing/inspection/cost/document/security impacts, history and actions.

The source preview respects access/classification policy. When preview is unavailable, preserve a locator and explain why; do not copy restricted text into an error message.

## Review flow

1. **Confirm package scope.** Verify part/assembly occurrence, model/drawing/spec/RFQ revisions, expected source checklist and unavailable references. Revision uncertainty is a blocker.
2. **Review proposals.** Imported semantic PMI or extraction suggestions are labeled `Proposed`, show form/support/warnings, and require accept, edit, reject-as-extraction-error, or request clarification. Graphical PMI is labeled `Visual evidence only`.
3. **Set applicability.** Choose applies, conditional, does not apply, or scope/revision uncertain. Non-applicability requires reason and approval identity.
4. **Resolve duplicate/conflict suggestions.** Side-by-side evidence shows why records appear similar or contradictory. Merge links lineage without deleting sources; conflict resolution records authority and basis.
5. **Cover impacts.** Link the requirement to manufacturing, inspection, documentation, security, lead-time and authoritative cost nodes, or record an explicit no-impact rationale. Warn about probable double allocation.
6. **Verify and assign.** Set owner, clarification status, verification/approval state, and any customer-facing assumption or exclusion.
7. **Evaluate readiness.** Run the pinned policy, inspect blockers/warnings, and publish an immutable evaluation.

Batch actions are limited to rows with the same source/type/applicability and no conflict/blocking warning. The preview lists every affected row and excludes exceptions; each persisted decision remains individually auditable.

## PMI-specific behavior

Selecting PMI synchronizes the model view, saved view/annotation plane, semantic detail and graphical presentation where available. The inspector separately shows:

- `semantic imported`, `semantic partially supported`, `graphical only`, `semantic/graphical conflict`, or `unsupported`;
- entity/value/unit/modifiers and geometry association;
- source AP/schema, importer version, warnings and human decision.

Missing graphical presentation does not invalidate supported semantics by itself, and attractive graphical presentation does not prove semantics. An unresolved or stale association cannot satisfy coverage.

## Blockers and exceptions

`Review blockers` groups issues by missing source, interpretation/applicability, conflict, impact/cost, approval, revision, and security. Each item explains the rule and remediation.

For an eligible blocker, `Request exception` opens a deliberate form: scope, reason, residual risk, customer visibility, approver, expiration/review point and affected outputs. The user must have permission; hard stops have no exception action and explain the controlling policy. Approval changes the evaluation to `Ready with exceptions` but leaves the source requirement visibly unresolved. Revocation/expiry affects only a new evaluation.

## Revision behavior

A new source/model revision creates a successor coverage set and a change inbox: added, removed, modified, conflicting, carried, or association-needs-review. The user accepts applicability/carry-forward per record. Prior coverage, exceptions and approvals remain read-only. Removed-source requirements remain visible until a reviewer confirms they no longer apply.

## Accessibility and safeguards

- All model highlights have equivalent source locators and textual geometry descriptions.
- Status is conveyed by text/icon/pattern, with high contrast and non-color legends.
- Table headers, filters, row actions, evidence tabs, conflict pairs and modal focus follow keyboard/screen-reader patterns; focus returns to the triggering row.
- Long lists support search and virtualized rendering without losing programmatic row position or selected state.
- Destructive-looking actions say what is retained (`Reject proposal; keep evidence`). Approval/exception actions show actor and scope before commit.
- Export, print, copy and screenshot affordances obey classification policy and show marking/recipient consequences.

## Completion and usability evidence

Completion requires a pinned source set, zero unexcepted blockers, recorded warnings/exceptions, reviewer identity, policy version and immutable result. It does not mean every possible requirement was discovered.

Usability tests use duplicates, conflicts, missing specifications, stale PMI associations, a graphical-only annotation, a no-cost requirement, and an ineligible security hard stop. Users must identify why the quote is blocked, trace requirement-to-cost evidence, distinguish semantic from graphical PMI, and understand that an exception is not resolution.
