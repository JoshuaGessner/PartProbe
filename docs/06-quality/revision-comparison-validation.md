# Revision Comparison and PMI Validation

> **Status:** In Review  
> **Last updated:** 2026-07-22  
> **Related requirements:** REQ-F-049–REQ-F-055, REQ-F-065; REQ-NF-015–REQ-NF-018, REQ-NF-020–REQ-NF-022; TEST-056–TEST-073  
> **Related ADRs:** ADR-0002–ADR-0008, ADR-0012  
> **Open questions:** Release thresholds by change class; independent ground-truth owners; authorized exporter fixtures; topology-mapping acceptance bands  
> **Dependencies:** Versioned fixture manifest, geometry/feature/calculation validation, NIST PMI cases, worker/security tests, reviewer UX harness  
> **Supersedes:** None

## Objective

Prove that comparison detects and explains supported changes without silently declaring unknown areas unchanged, and that PMI/requirements retain their form, source, applicability and human-review boundary. Tests establish behavior only for the named corpus, formats, translator versions and tolerances; they do not establish universal CAD or MBD accuracy.

## Evidence record

Each paired fixture pins:

- authorized source hashes, format/AP/schema/exporter metadata, revision/occurrence, classification and license;
- expected unit/frame/body/representation/validity/healing state;
- comparison, importer/kernel, feature, requirement, calculation, rate and policy versions;
- analytic or independently reviewed expected property, region, topology-correspondence, feature, PMI, requirement and cost deltas;
- allowed tolerance/ambiguity, known false positives/negatives, unsupported constructs and expected reason codes;
- expected visual region IDs, review actions, readiness result and audit events;
- OS/library fingerprint, elapsed/resource limits and derivative hashes.

Ground truth comes from analytically constructed parts, originating-CAD reviewed measurements/PMI exports, NIST public PMI test cases, or dual-tool evidence plus manufacturing/quality adjudication. A baseline generated only by the implementation under test is not independent truth.

## Required comparison matrix — TEST-056–TEST-060

| Case | Required assertions |
|---|---|
| Identical/same shape | byte-identical; differently serialized same geometry; entity/topology reorder; re-tessellation; repeated/cross-platform run. No false design change; pipeline-version differences remain separate. |
| Units/frames | equivalent mm/inch exports; declared unit-only change; intentional placement/orientation change; reviewed rigid alignment; ambiguous scale. Normalization cannot hide design placement or fabricate size change. |
| Local geometry | hole diameter/depth/location; pocket depth; thin wall; radius/chamfer; undercut/access; small high-impact change below coarse tessellation. Detect supported critical changes or return explicit incomplete/unknown. |
| Major/topological | added/removed body; split/merge faces/features; boolean-changing edit; assembly occurrence; healing-induced topology. Many-to-many/ambiguous mappings are not definite adds/removes. |
| Representation/failure | valid and invalid B-rep, mesh resolutions, cross-format where supported, open/non-manifold mesh, boolean failure, timeout/cancel/resource limit, malformed result. Confidence and stage states degrade safely; baseline/target remain unchanged. |

Global property equality cannot pass a local-change test: fixtures include offsetting added/removed volumes and changed hole position with unchanged volume. Visual tessellation cannot be the measurement oracle.

## Feature, manufacturing and cost assertions

- Compare reviewed feature snapshots separately from raw recognizer output; detect added/removed/modified/split/merge/reclassified and association-uncertain states.
- A changed importer/recognizer with unchanged source is `analysis_pipeline_delta`, not automatically a design change.
- Only accepted findings invalidate or seed downstream decisions; approved baseline routing and overrides remain byte-for-byte/logically unchanged.
- Target recalculation pins all inputs and versions. Component deltas exactly reconcile to target minus baseline for each quantity/currency.
- Design, routing, requirement, quantity, rate, pricing, capacity/policy and unknown deltas are separately classified; no double allocation.
- Plain-language explanations resolve to actual evidence and calculation nodes. Rounded display values reconcile through authoritative unrounded/fixed-precision values.
- Override carry-forward requires explicit recorded disposition and never inherits approval silently.

## Requirement coverage — TEST-061–TEST-065

Fixtures cover duplicate requirements from multiple sources; materially similar but non-duplicate requirements; direct conflicts; missing/expired/revision-mismatched references; no-cost rationales; one-to-many cost/inspection links; double-allocation risk; and security hard stops.

Tests assert orthogonal interpretation/applicability/coverage/verification states, immutable source lineage, correct blocker reason codes, and deterministic readiness under a pinned policy. An eligible authorized exception yields `ready_with_exceptions` and leaves the blocker unresolved. Unauthorized, expired or out-of-scope exceptions fail; non-waivable security/identity conditions expose no bypass. A new policy/source revision creates a new evaluation without changing an approved quote.

Automated extraction omission is tested with an intentionally unrecognized requirement: the UI must not claim source/package completeness merely because the proposal queue is empty.

## PMI/AP242 — TEST-066–TEST-069

Use NIST simple/fundamental/complex PMI cases plus authorized AP203/AP214/AP242 files from each supported exporter/profile. Include semantic-only, graphical-only, both-consistent, both-conflicting, missing associations, stale revision association, unsupported tolerance zone/modifier, datums/targets, dimensions/units, surface texture when claimed, saved views, material/product metadata, and malformed/partial transfer.

Assert:

- declared schema is recorded; AP/file extension does not imply content or completeness;
- semantic entities, value, unit, modifier, datum/reference and geometry association match independent expectations for the advertised subset;
- graphical presentation remains separate and rendered omissions/styling loss are reported;
- unsupported or dropped constructs produce counts/reason codes and cannot satisfy coverage;
- validation-property agreement is reported only as transfer evidence, never as semantic completeness or authority;
- PMI records remain bound to source/revision/importer version; re-import and revision mapping create successors/proposals;
- human accept/edit/reject preserves original evidence, confidence and warnings.

## Feature-level cost/risk visualization — TEST-070–TEST-073

- Selecting a reviewed feature/region resolves through the versioned selection map to its source geometry and shows feature type/dimensions, setup, operation, machine, candidate/selected tools, feeds/speeds source, path/removal method, cutting/support time, tooling/inspection cost, risk, confidence, corrections and allocated contribution when available.
- Every view mode—operation, setup, tool, machine, cutting/support time, cost, inspection, risk, confidence, accessibility, revision delta and removal intensity—reconciles to the same pinned snapshot. Picking, filtering and legend state cannot change authoritative values.
- Feature allocations reconcile to the estimate total or display an explicit unallocated remainder. Coarse/volumetric estimates use approximation labels and ranges; they never fabricate per-feature precision.
- Screenshot and interaction tests prove pattern/text alternatives to color, keyboard/list equivalence, screen-reader descriptions, selection stability under level-of-detail, policy-controlled export, and acceptable large-model responsiveness.

## Metrics and thresholds

Report per format/exporter/change class and representation—not one universal score:

- supported change precision/recall, false positive/negative counts and severity;
- property and added/removed-region error at named absolute/relative tolerances;
- topology/body/feature mapping exact, ambiguous and unmatched rates; split/merge accuracy;
- PMI supported-entity/value/unit/modifier/association recall and precision; unsupported/drop rate; graphical presentation coverage;
- requirement conflict/duplicate suggestion precision and readiness-blocker correctness;
- reconciled cost-delta error, unexplained remainder and explanation-link integrity;
- deterministic ordering/IDs/results, cross-platform numeric delta, time, peak memory, cancellation and incomplete rate;
- reviewer correction/acceptance time and critical-change discovery in usability studies.

Thresholds are approved per class before enablement. Any missed critical fixture change, silent unsupported PMI loss, false `ready`, unexplained authoritative cost delta, or unsupported high-confidence result is a release blocker even if aggregate scores pass.

## Robustness, security and privacy

Run malformed/polyglot/oversized/archive/external-reference and adversarial topology/PMI cases in the isolated no-network worker. Assert path containment, quotas, cleanup, cancellation, desktop survival and unchanged quote state. Diagnostics/logs contain codes and counts—not model/PMI text, coordinates, filenames, customer or price content. Cache, overlay, screenshot, export and test artifact classification inherits the stricter input; cross-project access/dedup inference is denied.

Only sanitized/public fixtures may enter ordinary CI. Authorized private fixtures run in an approved environment with access, retention and evidence-export rules; they are never uploaded to external services by default.

## Version, regression and release gate

Any importer/kernel, comparison, tolerance, mapping, feature, requirement, readiness, calculation or presentation change reruns its affected corpus. Golden changes require owner, cause, impact, reviewer and version/migration decision. Legacy result schemas remain readable or have a verifiable migration; approved historical outputs never recalculate in place.

Release requires three-platform package results; zero unexplained golden drift; all critical negative/failure/security cases; accessible textual equivalents for visual findings; manufacturing, quality and security reviewer sign-off; documented supported/unsupported PMI matrix; and published limitations. The product remains advisory until representative shop revision/quote outcomes support calibrated claims.
