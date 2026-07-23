# Revision Comparison Workflow

> **Status:** Draft  
> **Last updated:** 2026-07-22  
> **Related requirements:** REQ-F-049–REQ-F-050, REQ-F-053–REQ-F-055, REQ-F-065; REQ-NF-015–REQ-NF-018, REQ-NF-020–REQ-NF-022  
> **Related ADRs:** ADR-0001, ADR-0003, ADR-0007, ADR-0008, ADR-0012  
> **Open questions:** Default baseline; tolerance controls exposed to users; cost-delta approval roles; large-model performance target  
> **Dependencies:** Revision comparison architecture, model/feature review, requirement coverage, calculation graph, viewer/accessibility  
> **Supersedes:** None

## Goal

Let an estimator answer “what changed, what might that change in manufacturing, and what did it do to cost?” while preserving unknowns and every prior approved record. The workspace never equates a colored overlay with an approved design or quote change.

## Start and preflight

The user selects an immutable baseline and target by part/assembly occurrence, revision, source hash and snapshot state. The default baseline may be the latest approved quote for that occurrence, but the choice is explicit. Preflight shows units, coordinate frames, format/representation, body structure, validity/healing, PMI/feature/requirement availability, analysis versions and access classification.

Ambiguous units, wrong occurrence, unavailable source, unauthorized cross-project access, or incompatible scope blocks computation. A proposed rigid alignment is previewed with axes/transform/residual; the user confirms or rejects it. `Compare in source frames` remains available so alignment cannot conceal placement change.

## Workspace

- **Top summary:** baseline → target identity, comparison profile/version, review state, blockers/unknowns, accepted changes, projected internal cost/price/lead-time delta with context badges.
- **Left change tree/list:** identity, geometry, features, requirements, stock, setup, machine/tool, runtime, inspection, risk, cost/price/lead time. Filters include added/removed/modified/uncertain/unmatched and review status.
- **Center viewer:** side-by-side synchronized cameras by default; optional overlay, ghost, section and isolated-region views. Camera synchronization can be disabled and is announced.
- **Right inspector:** baseline/target values, method/tolerance, evidence/provenance, correspondence confidence, upstream/downstream impact path, history and review action.
- **Bottom impact/cost panel:** affected dependency nodes, current versus proposed decision, stale/unknown markers, selected recalculation and reconciled delta.

Added/removed/common/uncertain geometry uses label + outline/pattern + configurable color. A textual change list and model tree provide a complete alternative to spatial/color interpretation. The legend never says “unchanged” for uncomputed or unmatched regions.

## Review flow

1. Review identity, units, frame and global property deltas.
2. Walk localized regions and correspondence suggestions. Accept, reject or remap ambiguous baseline/target regions; split/merge mappings show all members.
3. Review feature and PMI/requirement changes. Distinguish `source design change` from `analysis/importer version difference` and `association uncertain`.
4. Inspect proposed stock/setup/machine/tool/runtime/inspection/risk impacts. Accepting a geometry match does not accept its manufacturing proposal.
5. Choose target routing changes and explicit override disposition: reapply and revalidate, replace, or do not carry.
6. Preview recalculation. Cost rows expose baseline/target calculation nodes and separate design, routing, quantity, rate, pricing, capacity/policy and unknown effects.
7. Save review. `Approve for estimate draft` creates/updates a target draft only with selected findings; it does not modify the baseline or approve a quote.
8. Run target requirement-coverage/readiness and normal quote approval before publication.

## Plain-language explanation

Every non-zero delta can expand to a chain such as:

> Pocket depth changed from 20 to 32 mm (reviewed exact-solid evidence). The selected routing adds two axial passes and a longer-reach tool. Runtime increases 14.2 minutes; at the pinned machine/labor rule versions, internal cost increases $38.40.

The explanation names assumptions and confidence. If rate or quantity also changed, it appears as a separate line. Unallocated remainder is explicit and blocks claiming a fully explained revision-cost delta.

## Feature-level cost and risk views

The reviewed target may be colored/filtered by operation, setup, tool, machine, cutting time, support time, cost contribution, inspection burden, risk, confidence, accessibility, revision cost change, or material-removal intensity. Selecting a feature shows its dimensions and geometry source; assigned setup/operation/machine; candidate and selected tools; feeds/speeds provenance; path/removal method; cutting/support time; tooling and inspection costs; risk/confidence; manual corrections; and allocated contribution.

These views render pinned estimate evidence—they do not calculate it. Coarse/volumetric estimates display broad regions and `Approximate / not allocable by feature`; any unallocated cost remains a named remainder. Legends show unit, quantity, scale/range, snapshot and confidence, and never imply more precision than the underlying method.

## PMI and requirement changes

Semantic, graphical and unsupported PMI deltas are shown separately. A missing target annotation is `possibly removed` until importer support and revision applicability are reviewed. Selecting a record shows its baseline/target geometry associations and source view. Accepted requirement changes flow to a successor coverage set as proposals and may block target quote readiness.

## History, failure and security

Prior model files, analysis/feature/coverage snapshots, approved routing, quote, pricing and overrides remain available read-only. Re-running with a new profile creates another result and identifies algorithm-driven changes.

Worker failure shows completed stages, missing evidence and retry/diagnostic options without clearing review work. Partial results cannot be approved as complete; an authorized reviewer may acknowledge named unknowns only through the applicable policy.

Both sides and every overlay/export inherit the stricter classification. Switching view modes does not leak the hidden side’s name/content. Copy, screenshot, print, export and support bundle actions follow policy and audit. Cross-customer comparison is denied by default.

## Accessibility and usability evidence

All viewport selection has list/tree equivalents; keyboard actions can step through changed regions; screen-reader labels state side, status, confidence and dimensions; focus returns to its change row; camera motion respects reduced-motion preference. Dense lists retain visible focus and programmatic position under virtualization.

Usability scenarios include no change, unit-only, intentionally rotated, one critical small hole/radius change, topology renumbering with no shape change, feature split/merge, graphical-versus-semantic PMI mismatch, rate-only delta and incomplete comparison. Users must not mistake uncertainty, analysis drift, or price-policy change for a design change.
