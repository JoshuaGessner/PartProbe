# CAM Reconciliation Validation

> **Status:** In Review  
> **Last updated:** 2026-07-22  
> **Related requirements:** REQ-F-058–REQ-F-059, REQ-F-065; REQ-NF-017–REQ-NF-021; TEST-079–TEST-084  
> **Related ADRs:** ADR-0005–ADR-0008, ADR-0014  
> **Open questions:** Shop CAM fixture rights, product/version matrix, time-basis truth source, and acceptable mapping/variance thresholds  
> **Dependencies:** Canonical CAM schema, adapters, reconciliation service, sanitized fixtures, actuals, and permissions  
> **Supersedes:** None

## Validation objective

Prove that CAM data can be imported, mapped, compared, and audited without vendor semantics leaking into the domain core, without confusing simulated time with machine/production actuals, and without changing approved estimate evidence. Passing tests demonstrates behavior for the named fixture versions only; it does not validate a CAM vendor generally.

## Test allocation

| ID | Scope | Required evidence |
|---|---|---|
| TEST-079 | Canonical contract and import | PartProbe JSON round-trip; CSV/XML/report adapter mappings; schema/version, source hash, units, time basis, unsupported fields, and deterministic canonical digest |
| TEST-080 | Partial, malformed, and hostile artifacts | missing/unknown fields, duplicate IDs, wrong units, oversized rows/nesting, malformed XML/CSV/JSON, active HTML/external assets, traversal references, timeout/crash, and sanitized recovery |
| TEST-081 | Setup/operation/tool/stock mapping | exact ID, reviewed map, heuristic, one-to-many, split/merge, ambiguous, unmatched, disabled operation, changed order, and no silent library promotion |
| TEST-082 | Time and quantity semantics | cutting/non-cutting/total, tool change/rapid/probe inclusion, prove-out, batch/part quantity, parallel work, zero baseline, mixed units, rounding, and simulation-basis differences |
| TEST-083 | Reconciliation and cause workflow | estimate↔CAM, CAM↔machine, and machine↔actual comparisons; partial stages; reviewer-confirmed/split/unknown causes; exact cost/time category coverage without double counting |
| TEST-084 | Lifecycle, version, security, and audit | re-import, adapter/schema/post/CAM upgrade, mapping change, approval/supersession, replay/migration, permissions/classification, export, no-network/report-fetch, and immutable approved estimate/actuals |

## Fixture families

Each golden fixture manifest records ownership/license/classification, source product/version or synthetic generator, report/template/post version, source hash, expected canonical records, expected unsupported fields/warnings, mapping ground truth, time/quantity basis, estimate/routing snapshot, optional machine/actual snapshot, adapter/schema version, reviewer, and tolerance.

Minimum families:

1. Synthetic PartProbe canonical interchange with exact complete mappings.
2. One sanitized shop-selected CAM export with multi-setup machining, repeated tools, suppressed operation, and setup-sheet metadata.
3. A structurally different vendor/report or independently generated neutral fixture proving the core does not require the first adapter's fields.
4. Partial report lacking one or more optional time/tool/stock fields.
5. Malformed/hostile corpus containing active HTML references, oversized/deep data, encoding/locale/decimal variants, duplicate IDs, and path-like values.
6. Revision pair where an operation is split/merged/reordered and adapter version changes.

Customer/controlled artifacts are never copied into a public test suite. Private fixtures use authorization, encrypted/approved storage where policy requires, least privilege, content-minimized test output, and a reproducible synthetic/public companion where possible.

## Invariants and oracles

- Original artifacts and hashes remain unchanged.
- Canonical serialization/digest is deterministic across Windows, Linux, and macOS for the same validated input and adapter version.
- Every authoritative field has source, units/type, and adapter/schema version.
- `Estimated`, `CamSimulated`, `MachineObserved`, and `ProductionActual` cannot be interchanged by serialization or UI mapping.
- Unknown/missing/ambiguous values propagate as typed states; no missing time, quantity, cost, tool, setup, or mapping becomes zero or matched.
- Each setup/operation/tool/stock source record is matched, explicitly grouped/split, not applicable, or visibly unmatched.
- Each compared occurrence contributes once to its applicable time/cost category; relative variance is undefined at a zero base.
- Heuristic mappings and variance causes remain proposals until reviewed.
- Re-import, remap, or new adapter/policy creates a new snapshot; old approvals and quotes replay unchanged.
- Vendor-specific fields can be added in an adapter extension/evidence payload without adding vendor types to core aggregates.

## Security and failure tests

Run all adapters with authorized sanitized fixtures and the same trust posture as other imports. Assert file/row/entity/depth/memory/CPU/wall-time/output limits, cancellation, staging cleanup, path containment, no external HTML asset/script execution or fetch, no arbitrary file/network access, and content-minimized errors/logs. Fuzz parsers and canonical DTO validation. Test deny-by-default project/classification/record-state permissions for import, mapping, viewing NC/setup artifacts, approval, export, and deletion/disposition.

Adapter/API credential and network tests are required only when such an adapter is proposed, but then include destination allowlist, minimum-data request, revoked/expired credential, retry/idempotency, rate limit, partial response, audit, and proof that technical data is not sent to an unapproved destination.

## Numerical and semantic checks

Use exact decimal comparisons for currency and documented tolerances only for physical/time values whose source warrants them. Test unit conversion, locale decimal separators, time-zone metadata, seconds/minutes/hours, per-tool/per-operation/per-setup/per-part/per-batch totals, make versus deliver quantity, and simulation components included in vendor totals. Never force component sums to equal a vendor total when the vendor basis is undocumented; record the discrepancy and block the affected interpretation.

Golden variance examples cover tool mismatch, extra/missing setup, stock change, feed/speed change, CAM simulation versus estimated cutting/non-cutting, machine delay, inspection burden, rework, and unknown cause. Expected category mappings and no-double-count rules are reviewer-authored.

## Release gates

Block the relevant adapter/reconciliation release for source mutation, silent data loss, guessed units/time basis, mismatched project/model acceptance, uncontrolled network/file access, content in diagnostics, authorization bypass, vendor type leakage into core, non-deterministic replay, incorrect category double counting, or mutation of approved estimate/actuals.

Initial acceptance permits only manual/canonical and explicitly validated file adapters. Each new CAM product, major version, report template, post family, API, or machine-observation source requires fixture impact review and adapter conformance evidence. Accuracy/coverage thresholds remain shop-approved and must report both match precision and completeness; a high match percentage cannot hide unmatched critical operations.

