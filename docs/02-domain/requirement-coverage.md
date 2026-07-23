# Manufacturing Requirement Coverage

> **Status:** In Review  
> **Last updated:** 2026-07-22  
> **Related requirements:** REQ-F-051–REQ-F-054, REQ-F-065; REQ-NF-015–REQ-NF-018, REQ-NF-020–REQ-NF-022  
> **Related ADRs:** ADR-0006–ADR-0008, ADR-0012  
> **Open questions:** Which requirement classes are always blocking; who may approve exceptions; source precedence by customer; minimum evidence for “not applicable”  
> **Dependencies:** Quote lifecycle, quality/inspection model, PMI research, permissions matrix, security policy  
> **Supersedes:** None

## Purpose

The coverage model proves what the estimate team recognized, interpreted, costed, planned, or deliberately excepted. It does not claim that automated extraction found every obligation. Coverage belongs to an exact RFQ/input-package and part/model revision set; an approved quote pins its evaluated snapshot.

## Aggregate and lifecycle

```text
RequirementCoverageSet
 ├─ RequirementSource [1..n]
 ├─ ManufacturingRequirement [0..n]
 │   ├─ SourceEvidence [1..n]
 │   ├─ ApplicabilityDecision [1..n]
 │   ├─ ImpactLink [0..n]
 │   └─ VerificationDecision [0..n]
 ├─ RequirementConflict [0..n]
 ├─ QuoteReadinessEvaluation [0..n]
 └─ ReadinessException [0..n]
```

`RequirementCoverageSet` is a versioned working aggregate until published. Publication creates an immutable snapshot. New source bytes, source revision, extraction rule, user decision, or applicability scope creates a successor; approved quotes continue to reference the prior snapshot.

## Core records

| Record | Required content |
|---|---|
| `RequirementSource` | stable ID; source kind; asset/document/RFQ ID and revision; SHA-256; received and effective dates; classification; page/view/entity/field locator scheme; extraction availability and completeness state |
| `ManufacturingRequirement` | stable ID; verbatim or losslessly referenced source text/value; normalized type and value; category; criticality; part/assembly and revision applicability; interpretation state; owner; provenance; open question/customer clarification; approval state |
| `SourceEvidence` | exact source locator, excerpt/value hash, extraction method/version, semantic/graphical/user-entered form, confidence and warnings; sensitive text is not copied into ordinary logs |
| `ImpactLink` | requirement version to routing operation, estimate component, inspection task, document/deliverable, supplier action, security control, lead-time constraint, or explicit `no_cost_impact` rationale |
| `RequirementConflict` | involved requirement/source versions, conflict type, blocked decisions, owner, resolution/authority and time; no source is deleted when resolved |
| `QuoteReadinessEvaluation` | coverage snapshot, quote/estimate revision, policy/rule version, blockers, warnings, exceptions, outcome, evaluator/time |
| `ReadinessException` | blocker IDs, scope, reason, authority/role, approver/time, expiry/review point, residual risk, customer visibility and status |

Stable IDs identify requirement lineage; immutable versions identify exact content. A duplicate detector may link records, but merging requires review and retains every source locator.

## Sources and categories

Sources include exact model geometry, semantic or graphical PMI, drawings, general notes, referenced specifications, purchase-order/RFQ clauses, customer manuals/portals, structured RFQ fields, material/process specifications, user entry, and versioned historical customer rules. An unavailable referenced document is itself a coverage issue; its unseen requirements must not be invented.

Categories include material/condition; dimensional tolerance and GD&T; finish/threads; heat treatment; coating/plating/anodize/passivation; NDT/welding/cleaning; marking/serialization/traceability; material/process certifications and certificate of conformance; FAI/partial FAI/CMM/sampling/capability/source inspection; customer approval; packaging/preservation/shipping; record retention; and security/export/customer-proprietary handling. Legal/export classifications remain authorized determinations, not estimator inference.

## Orthogonal states

Do not collapse interpretation, applicability, coverage, and verification into one checkbox.

| Axis | Allowed states |
|---|---|
| Interpretation | `unreviewed`, `proposed`, `confirmed`, `needs_clarification`, `rejected_as_extraction_error` |
| Applicability | `applies`, `does_not_apply`, `conditional`, `revision_uncertain`, `scope_uncertain` |
| Coverage | `recognized_and_costed`, `recognized_no_cost_rationale`, `recognized_not_costed`, `deferred`, `missing_source_information`, `potential_conflict` |
| Verification | `not_planned`, `planned`, `evidence_pending`, `verified`, `not_applicable` |
| Approval | `draft`, `reviewed`, `approved`, `superseded` |

UI summary labels may simplify these combinations, but persisted records retain each axis and reason. `does_not_apply`, `no_cost_rationale`, and `deferred` require an actor, time, and justification.

## Requirement-to-impact contract

A confirmed applicable requirement is covered only when its required impacts are linked or explicitly justified:

- manufacturing: operation/setup, outside process, approved supplier class, tooling/fixture, or material/stock decision;
- inspection: characteristic/method/equipment/skill/sampling/FAI task and expected effort;
- documentation: certification, traceability, marking, retention, and customer-deliverable work;
- security: classification, authorized environment/recipient, export/print/share restrictions;
- commercial: estimate components, risk/contingency where policy permits, lead-time constraints, and quote assumptions/exclusions.

Cost linkage names an authoritative calculation node/result version. It must not duplicate cost already allocated through another requirement or operation. Many-to-many links include an allocation/rationale and reconciliation status.

## Quote-readiness rule

An evaluation runs against an immutable coverage snapshot and versioned policy. The result is one of `not_ready`, `ready_with_warnings`, `ready_with_exceptions`, or `ready`.

The estimate is not ready when any applicable blocker has:

- unreviewed or unresolved interpretation/applicability;
- missing referenced source information;
- unresolved credible conflict;
- required manufacturing, inspection, documentation, security, cost, or lead-time impact without a link/rationale;
- a source/revision mismatch, stale geometry association, invalid model-unit decision, or incomplete critical import;
- a policy hard stop such as unsupported classified information or unauthorized processing.

Warnings do not become blockers merely because they are numerous; severity comes from a versioned policy with reason codes. A later policy evaluation creates a new result and never rewrites an approved quote.

### Exceptions

Only a permissioned user may except an eligible blocker. The exception records the unresolved fact and residual risk; it never changes that requirement to resolved. The UI and outputs say `ready_with_exceptions`, list customer-visible assumptions/exclusions as policy requires, and preserve approval evidence.

Hard-stop security/authorization conditions, an unknown quote identity/revision, or missing approval authority cannot be excepted inside the estimate workflow. Policy defines any additional non-waivable classes. Expired or out-of-scope exceptions restore the blocker automatically on the next evaluation without altering historical results.

## PMI and automated extraction

Semantic PMI produces proposed requirements only when the importer reports support for the entity, value, units, modifiers, geometry association, and source revision. Graphical PMI and document extraction are visual/text evidence, not semantic confirmation. Missing PMI never means “no tolerance,” and imported PMI cannot demonstrate that a source package is complete. Human review records the disposition while preserving system confidence and warnings.

## Revision and conflict rules

- Applicability is explicit to part/assembly occurrence and revision/range; “latest” is not persisted as authority.
- A new model/drawing/specification revision creates a new coverage proposal and marks affected links `needs_review`.
- Requirements may be carried forward only with recorded source continuity and reviewer approval.
- Geometry and requirement comparison propose impact; they do not overwrite an approved routing, inspection plan, estimate, exception, or quote.
- Source precedence is policy/context-specific. Conflicting credible evidence blocks rather than silently choosing a winner.

## Security, retention, and audit

Requirements and derived snippets inherit the strictest source classification. Search results, exports, notifications, diagnostics, screenshots, and support bundles enforce the same project/classification boundary. Access is checked at the service/repository boundary. Audit events cover source ingest, extraction, merge/split, interpretation, applicability, impact links, conflict resolution, readiness evaluation, exception, approval, export, and supersession.

Retain source hash and locator even if policy later disposes source content; retention behavior must not falsely imply the disposed requirement can still be reverified. No controlled source is sent to external AI/OCR or validation services by default.

## Staged scope

1. **Foundation:** manual source registry, manually entered requirements, orthogonal states, impact links, blockers/exceptions, immutable readiness result.
2. **Reviewed assistance:** AP242 PMI proposals, duplicate/conflict suggestions, revision applicability prompts, cost/inspection link reconciliation.
3. **Advanced:** governed document extraction, customer-rule libraries, richer specification dependencies, and coverage analytics. None may bypass human confirmation.

The first vertical slice may expose a simple checklist and blocker summary but should not claim complete automated extraction. Canonical acceptance is defined by TEST-061–TEST-069 and the dedicated workflows/validation plans.
