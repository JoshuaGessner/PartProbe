# USE-3 Stock Proposal Validation

> **Status:** In Review
> **Last updated:** 2026-09-10
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

Full local closeout passes 277 runtime tests plus one compile-fail doctest. Strict workspace/native-host/WASM Clippy, the offline release frontend build, 74 Python tooling tests, 69 fixture hashes, 149-file planning validation, formatting, and diff hygiene pass. The UI regression pins the complete Huntsville test-profile values and proves confirmations remain clear. Ten adapter tests and eleven supervised native worker tests pass against the fresh pinned Apple-Silicon install. The desktop host passes 42 ordinary tests with four configured native/package smokes ignored by default. The final 181 MiB unsigned arm64 contract-v10 package has a completely reverified 72-artifact runtime and passes two opt-in real-STEP package smokes: the retained manual result and the proposal/adoption path with exact 12×8×5 mm model bounds, 15×11×7 mm blank, 1,155 mm³ stock, 675 mm³ removal, material conversion, and explicit exclusion trace. Its GUI executable SHA-256 is `6128973fe0ec5acb5535dcc9a8e37a22f8b50b943224356661615274db6359b4`; its worker SHA-256 is `4f7ab1803d15186917dce4a4eec0e8c7821fa8809bddc9c274f345faaa12d4c9`. Live macOS inspection verifies the cleaned Estimate intake and the explicitly unconfirmed Huntsville Rates/Resources screens; a complete human STEP-to-result rehearsal remains pending before the industry live test.

## Version and migration decision

These are new, non-persisted internal proposal/evidence/adoption contracts. No historical estimates, quotes, settings rows, geometry snapshots, or customer records are migrated or reinterpreted. `geometry-snapshot-v1` remains byte-for-byte unchanged and remains accepted as an explicit envelope-unavailable legacy result. Adapter ABI v4 replaces v3 by adding the three bounds values; mixed worker/adapter binaries fail their ABI/size checks and must be rebuilt together. The new worker output reference is additive, so legacy stored exact outputs remain explicitly envelope-unavailable rather than being upgraded. Contract v10 adds path-free fields/one command without migrating persistence. The developer proposal/adoption rules are both v1.0.0 and apply only to newly requested session calculations; prior manual results are not recalculated. Current ABI-v4 evidence is Apple-Silicon only; Linux, Windows, and release evidence must be refreshed before broader claims. Any derivative/container field, source-binding rule, bounds algorithm/tolerance, body eligibility, formula, allowance semantics, orientation, excluded-input set, result field, confidence, or diagnostic change requires a new version and an explicit migration/replay decision.
