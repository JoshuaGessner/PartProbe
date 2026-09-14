# USE-3 Stock Proposal Validation

> **Status:** In Review
> **Last updated:** 2026-09-13
> **Related requirements:** REQ-F-005, GEO-008, GEO-011, CALC-002–CALC-007, TEST-027
> **Related ADRs:** ADR-0002, ADR-0004
> **Open questions:** Shop-approved allowances, standard-size catalogs, orientation policy, availability source
> **Dependencies:** `stock-selection-model.md`, USE-2 active proposal-selection evidence
> **Supersedes:** None

## Scope

`crates/setup-planner` implements the first pure USE-3 rule, `partprobe-rectangular-stock-envelope-policy` v1.0.0. The rule is UI-, storage-, and CAD-kernel-independent. Geometry-core owns additive exact STEP envelope derivative schema v1 with positive source-axis AABB extents in canonical millimetres and the source SHA-256. Geometry-import's `geometry-step-analysis-v1` container carries the complete unchanged snapshot-v1 plus that derivative. Setup-planner consumes the validated container and supervisor-owned controlled-output SHA-256.

Native OCCT adapter ABI v4 measures the additive AABB from already-authorized exact STEP bytes using `BRepBndLib::AddOptimal(shape, box, false, false)`. This explicitly ignores triangulation and shape-tolerance inflation, rejects void/open/non-finite/non-positive bounds, and passes three canonical-millimetre extents into the worker's `geometry-step-analysis-v1` result. Direct path/byte adapter tests and supervised-worker tests reproduce `FIX-STEP-001` and `FIX-STEP-003` within their committed 0.000001 mm tolerance. A fresh pinned Apple-Silicon OCCT source build, native verifier, manifest-bound runtime, configured desktop-host STEP smoke, and package-contained proposal/adoption smoke pass. This establishes local developer-native extraction, application retention, and explicit session adoption—not supported import, three-OS ABI-v4 evidence, production estimating accuracy, or quote authority.

## Governed behavior

The rule requires exactly one solid body plus an `active_for_proposals` selection that pins the exact supplied stock-allowance identity/version. The allowance must be `Reviewed` or `Approved` and `Rectangular`. X/Y/Z allowances are total dimensional additions, applied once per axis. Checked decimal arithmetic calculates blank dimensions and volume, then subtracts exact part volume. Multi-solid analysis, part volume larger than the model envelope, arithmetic overflow, negative removed volume, wrong selection/state/version, Draft/retired evidence, and round/plate forms fail visibly. This prevents one aggregate bounding box from being described as one machinable part.

Every successful v1 result remains `NeedsReview` with three stable reasons: only source-axis orientation was considered; no standard size was resolved; and availability was not resolved. The result pins source/output hashes, selection and allowance versions, model extents, allowances, blank dimensions, blank volume, removed volume, and rule identity/version. It does not resolve material, density, price, workholding, saw/facing strategy, reservation, or estimate adoption.

## Executable evidence

Seven focused tests cover:

- the 10×10×10 mm cube with 1,000 mm³ part volume and 3/3/2 mm total allowances, producing a 13×13×12 mm blank, 2,028 mm³ blank volume, and 1,028 mm³ removed volume;
- the independent 12×8×5 mm prism with 480 mm³ part volume under the same policy, producing 15×11×7 mm, 1,155 mm³ blank volume, and 675 mm³ removed volume;
- distinct proposal values for those distinct governed model dimensions;
- rejection of a non-active selection and a mismatched stock-policy identity/version;
- rejection of Draft evidence and an unsupported round form;
- rejection when part volume exceeds the supplied model envelope; and
- rejection of a multi-solid exact analysis before proposal calculation.

Seven additional contract/application tests prove positive canonical derivative validation, false-authority/zero rejection, unchanged snapshot-v1 retention, embedded and authorized source mismatch rejection, application-session retention, exact active-catalog resolution with output-hash preservation, and explicit unavailability for legacy STEP or inactive catalog evidence.

The additive internal live-test path adds seven application tests. They prove exact draft-library/source/child pinning; model-sensitive cube/prism blank cost and cutting time; kg/m³-to-kg/mm³ conversion; milling and machine-envelope gates; visible missing/legacy/lifecycle/form failures; explicit proposal and limitation review; quantity-sensitive purchased material; recorded actor/time/reason; and exact zero only for the six named exclusion groups after affirmative review. Desktop contract v10 adds one exact permissioned proposal command, path-free proposal/adoption DTOs, source-axis bounds, settings/library optimistic pins, and a result adoption/exclusion trace. The UI retains the complete manual path, labels STEP estimating versus STL/3MF analysis, and offers the documented Huntsville test profile only through a user-invoked unsaved form action that still requires review and confirmation.

The 2026-09-13 contract-v12 usability correction makes that unsaved boundary explicit before proposal invocation. Confirmed form values no longer make **Prepare estimate inputs** appear ready: without a persisted revision the action reads **Open settings to save**, routes to Settings, and labels the form **Save required**. The native fallback distinguishes an empty repository with `USE2-SETTINGS-DRAFT-MISSING` and an actionable save message from actual storage unavailability. Tests pin the empty-repository diagnostic and the WebView saved-draft gate. No calculation, proposal, settings schema, command set, or persisted evidence changed.

The subsequent internal-test workflow correction preserves that persistence boundary while removing the misleading repeat work. **Install Huntsville test profile** is now one explicit action that loads the documented values, supplies the fixed local-test actor/reason, confirms the visibly summarized test-only basis, saves the immutable first revision, and returns to Estimate. A later application launch shows the saved revision and requires one **Use saved profile** confirmation for the session; a successful save no longer clears its own session confirmation. After exact STEP analysis and a saved revision, proposal preparation begins automatically. Analysis units/warnings and proposal/limitations each use one combined explicit confirmation, the review action supplies a visible default reason that remains editable, and one explicit checkbox records zero spares/destructive samples. Native calculation/adoption requirements, settings revision pins, named exclusions, and missing-versus-zero behavior are unchanged.

`proposed-stock-display-placement-v1` is a new display-only interpretation of the unchanged proposal result. The native host parses the matching proposal's three positive blank dimensions and passes them directly to the native viewer state; no new command or WebView geometry field exists. The renderer verifies that the dimensions enclose the retained scene bounds, centers the source-axis blank on those bounds, and thereby splits each v1 total-axis allowance equally across opposing faces. A source or saved Settings change clears the layer. Unit tests cover positive rendering buffers, enclosure rejection, and path-free review/availability notices. This does not change stock calculation, estimate adoption, standard-size/availability, purchasing, workholding, routing, or CAM authority.

The fresh 203 MiB unsigned arm64 package passes package-contained real-STEP proposal/adoption and source-scene smokes with the unchanged embedded worker. A direct launch with no environment or viewer flag analyzed the governed 10 mm cube, automatically prepared a 16.4 × 16.4 × 16.4 mm blank from saved Settings revision 2, completed both combined reviews plus the explicit zero-additional-pieces action, and returned USD 236.75. The same-window viewer showed that proposal blank around the lit model. This is internal workflow evidence for the documented test profile, not a market-validated price or supported arbitrary-part estimate.

The earlier contract-v10 checkpoint closed at 277 runtime tests and a 181 MiB package. It remains historical evidence for the initial proposal/adoption boundary and has been superseded for current GUI/package status by the contract-v12 package and direct-launch rehearsal above.

## Version and migration decision

These are new, non-persisted internal proposal/evidence/adoption contracts. No historical estimates, quotes, settings rows, geometry snapshots, or customer records are migrated or reinterpreted. `geometry-snapshot-v1` remains byte-for-byte unchanged and remains accepted as an explicit envelope-unavailable legacy result. Adapter ABI v4 replaces v3 by adding the three bounds values; mixed worker/adapter binaries fail their ABI/size checks and must be rebuilt together. The new worker output reference is additive, so legacy stored exact outputs remain explicitly envelope-unavailable rather than being upgraded. Contract v10 adds path-free fields/one command without migrating persistence. The developer proposal/adoption rules are both v1.0.0 and apply only to newly requested session calculations; prior manual results are not recalculated. Current ABI-v4 evidence is Apple-Silicon only; Linux, Windows, and release evidence must be refreshed before broader claims. Any derivative/container field, source-binding rule, bounds algorithm/tolerance, body eligibility, formula, allowance semantics, orientation, excluded-input set, result field, confidence, or diagnostic change requires a new version and an explicit migration/replay decision.
