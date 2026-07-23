# CAM Reconciliation Architecture

> **Status:** In Review  
> **Last updated:** 2026-07-22  
> **Related requirements:** REQ-F-058–REQ-F-059, REQ-F-065; REQ-NF-017–REQ-NF-021; TEST-079–TEST-084  
> **Related ADRs:** ADR-0005–ADR-0008, ADR-0014  
> **Open questions:** Shop CAM systems/versions, report/API formats, identifiers, machine-time sources, and data-transfer policy  
> **Dependencies:** Integration boundary, actuals, routing/runtime model, document storage, permissions, and adapter discovery  
> **Supersedes:** None

## Objective and non-goals

The pipeline connects `Estimate → CAM plan → Machine result → Final actual cost` while preserving that each stage has a different authority and uncertainty. A CAM simulation is not an estimate or an actual; NC metadata is not proof of execution; machine telemetry is not a complete job-cost record. Reconciliation explains differences and feeds reviewed calibration/learning evidence.

PartProbe does not generate, edit, post, approve, transfer, or execute NC code in this boundary. It does not require one CAM vendor and does not treat parsing G-code as a reliable universal substitute for structured CAM context.

## Ports and adapters

The core owns a versioned canonical reconciliation contract and these ports:

- `CamArtifactIngestPort`: stages source artifacts, validates manifest/schema, hashes originals, and returns typed candidate records.
- `CamAdapter`: maps a named vendor/report/version into canonical records; it never returns persistence rows or domain decisions.
- `CamMappingService`: proposes links from imported setups/operations/tools/stock to estimate/routing records with evidence and confidence.
- `CamReconciliationService`: compares pinned snapshots and emits typed variance records; users resolve uncertain mappings and causes.
- `MachineResultPort`: optionally imports authorized machine-cycle observations through a separate adapter.
- `ActualsPort`: references final production/job-cost actuals without changing their source authority.

Initial imports are local files or manual entry. Candidate formats are a PartProbe JSON interchange, CSV, XML, and vendor-generated reports/setup sheets. Vendor API/plugins remain optional adapters outside the domain core. Autodesk Fusion, for example, can generate HTML setup sheets for selected setups, NC programs, or operations, illustrating why report-specific adapters are needed rather than assuming one universal report schema ([Autodesk Fusion setup-sheet documentation](https://help.autodesk.com/cloudhelp/ENU/Fusion-CAM/files/MFG-SETUP-SHEET-STEPS.htm)).

MTConnect may later normalize authorized equipment observations, but it is a machine-equipment information model and agent protocol—not a CAM plan contract or actual-cost ledger. Its official materials describe a vendor-neutral semantic vocabulary and versioned machine-readable model ([MTConnect overview](https://www.mtconnect.org/), [standard downloads](https://www.mtconnect.org/standard-download20181)).

## Canonical records

`CamImport` is immutable and stores source artifact hash/locator/classification, CAM product/version, postprocessor/version where present, adapter/schema version, import time/actor, project/model identifiers, units/time basis, warnings, unsupported fields, and source manifest.

Canonical child records include:

- `CamSetup`: stock/work offset/orientation, machine, fixture/workholding, setup name/order, images/notes references.
- `CamOperation`: vendor/native ID, setup, type/strategy, geometry or feature references when supplied, tool, parameters, order, enabled/suppressed state, simulated cutting/non-cutting/total time, and simulation basis.
- `CamTool`: vendor tool identity, assembly/holder/cutter properties, offsets, and mapping candidate; not silently promoted into the shop library.
- `CamStock`: material/form/dimensions/orientation/allowance with source units.
- `CamProgramMetadata`: NC program/post/machine-control identifiers, generated time, and statistics; NC bytes remain a restricted artifact if retained.
- `CamMapping`: source and target IDs, status (`Matched`, `Proposed`, `Ambiguous`, `Unmatched`, `Not applicable`), method/evidence, confidence, reviewer, and version.
- `CamReconciliation`: exact estimate/routing/CAM import/machine-result/actual snapshot IDs, mapping set, policy version, comparisons, warnings, review state, and digest.
- `VarianceRecord`: category, estimate/CAM/machine/actual values with units and time bases, absolute/relative difference where defined, cause, evidence, confidence, owner, and disposition.

Large setup images, reports, NC artifacts, and attachments remain in the classified content-addressed store. Searchable normalized metadata stays in persistence. Unknown vendor fields are preserved in the immutable source, not accepted as authoritative canonical values.

## Matching and comparison

Matching proceeds from stable explicit IDs, then reviewed mapping tables, then typed attributes/order/name heuristics. Heuristic matches are proposals. Ambiguous or one-to-many mappings require review. Operation aggregation/splitting is explicit so comparisons cannot manufacture one-to-one correspondence.

Comparisons include setups, operations, tools, stock, machine, programming/setup/cutting/non-cutting/cycle time, material, inspection, scrap, and cost where each stage supplies an applicable value. Every value retains its basis:

- `Estimated`
- `CamSimulated`
- `MachineObserved`
- `ProductionActual`

Time basis records scope, quantity, warm-up/prove-out, rapid/tool-change/probe behavior, overrides, simulation accuracy settings, post/machine/control assumptions, and whether parallel/unattended time is included. Relative variance is undefined when the comparison base is zero.

Variance causes use a versioned taxonomy: geometry analysis, feature recognition, setup planning, tool selection, feeds/speeds, runtime model, programmer decision, machine limitation, operator delay, tool failure, inspection burden, material issue, rework, outside process, schedule disruption, mapping uncertainty, and unknown. The system suggests at most; a reviewer confirms cause and may split a variance across causes.

## Lifecycle and immutability

`CamImport`: `Staged → Validated → Mapped → Frozen`, or `Rejected/Quarantined/Superseded`.

`CamReconciliation`: `Draft → Mapping review → Variance review → Approved`, then `Superseded` by a sibling comparison. Re-importing a changed report or running a newer adapter creates a new import and mapping set. Approved estimates, CAM imports, reconciliations, and actuals never mutate each other.

Every result pins source hashes; CAM/adapter/schema/post versions; estimate, routing, runtime, calculation, tool/library, and actuals versions; mapping/policy versions; actor/time; and warnings. A missing stage remains `Unavailable`; the pipeline supports estimate-to-CAM before machine/actual data exists.

## Security and operational controls

Treat CAM reports, setup images, toolpaths/NC metadata, models, prices, and machine/job identifiers as project technical data. Import is local/offline by default and follows bounded staging, allowlisted types, safe path handling, size/row/depth/time limits, source hashing, isolated parser rules where warranted, and sanitized diagnostics. HTML setup sheets are parsed as untrusted data; active content and external resources are never rendered or fetched.

Adapters receive only authorized artifacts and mapping context. No adapter may upload content, invoke a public CAM service, or read arbitrary project/filesystem data. External/vendor APIs require separate destination/classification approval, credential handling, egress audit, retry/idempotency, retention, and revocation policy. NC transfer/execution is out of scope. Audit import, rejection, mapping decisions, cause changes, approval, export, and deletion/disposition.

NIST's manufacturing-data recommendations explicitly address file and stream traceability/trustworthiness, supporting preserved artifacts, provenance, and open canonical boundaries rather than lossy vendor-specific persistence ([NIST AMS 300-10](https://www.nist.gov/publications/recommendations-ensuring-traceability-and-trustworthiness-manufacturing-related-data)).

## Failure behavior

- Partial imports retain valid staged records with unsupported/missing fields listed; they cannot be frozen if mandatory identity/unit fields are absent.
- Unknown units or time basis block affected comparisons rather than guessing.
- Adapter crash/timeout produces a recoverable failed import and cannot alter an approved reconciliation.
- Duplicate artifact hashes are detected; user may link the existing import rather than duplicate evidence.
- Source/adapter version mismatch requires explicit compatibility handling.
- Unmatched records remain visible and prevent a completeness claim; they are never dropped to improve variance.

## Staged release

- **Early foundation:** canonical schema, immutable source manifest, manual mapping, and manual reconciliation.
- **Initial production:** no CAM dependency; actuals remain usable without CAM.
- **Intermediate:** local CSV/JSON/XML/report import for one or two shop-selected formats, estimate-to-CAM comparisons, reviewed variance causes.
- **Advanced:** maintained vendor adapters/APIs, machine-observation adapters, CAM-to-machine-to-actual reconciliation, and governed learning feeds.
- **Deferred/research:** NC semantics inference, writeback/add-ins, broad vendor coverage, and any cloud content exchange.

## Acceptance boundary

The architecture is ready for implementation only after the shop supplies sanitized representative exports, identifies products/versions and time semantics, and approves the data boundary. Validation is defined in [CAM reconciliation validation](../06-quality/cam-reconciliation-validation.md). Passing one vendor fixture does not make the canonical contract vendor-neutral; at least two structurally different fixture families or one vendor plus the PartProbe interchange must exercise the core without domain changes.

