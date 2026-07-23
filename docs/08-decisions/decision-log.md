# Decision Log

> **Status:** In Review  
> **Last updated:** 2026-07-22  
> **Related requirements:** All  
> **Related ADRs:** ADR-0001–ADR-0014  
> **Open questions:** ADR acceptance owners/dates  
> **Dependencies:** Spike evidence  
> **Supersedes:** None

## Working decisions

- **2026-07-22:** Adopt documentation-first Phase 0 governance and stable ID conventions. Status: Active.
- **2026-07-22:** Treat the product as a human-reviewed estimator, not autonomous quote generation or production CAM. Status: Active product boundary.
- **2026-07-22:** Treat local/offline and no external technical-data upload as the default boundary. Status: Active security principle.
- **2026-07-22:** Preserve source files, hashes, analysis versions, approved snapshots, and override history. Status: Active invariant.
- **2026-07-22:** Advanced analyses wrap the deterministic estimate as versioned, explainable proposals; they do not silently replace the baseline or human authority. Status: Active product boundary.
- **2026-07-22:** Keep theoretical capability, availability/readiness, accounting cost, contribution/opportunity, and selling price as distinct concepts. Status: Active domain invariant.

## Architecture decisions awaiting evidence

| ADR | Proposal | State | Evidence required before acceptance |
|---|---|---|---|
| ADR-0001 | Tauri 2 + Leptos CSR | In Review | 10-day UI/platform/accessibility/security spike |
| ADR-0002 | OCCT exact B-rep kernel | In Review | Corpus accuracy, FFI/packaging/license evidence |
| ADR-0003 | wgpu model rendering | In Review | Picking/overlay/performance/platform spike |
| ADR-0004 | STEP/STL/3MF first-class; IGES/OBJ experimental | In Review | Import corpus and packaging evidence |
| ADR-0005 | Isolated geometry worker | In Review | IPC, crash/resource containment and operations evidence |
| ADR-0006 | SQLite + content-addressed local blobs | In Review | Migration/crash/backup/restore/concurrency threat spike |
| ADR-0007 | Typed versioned calculation DAG + fixed decimal | In Review | Exact golden/property/serialization evidence |
| ADR-0008 | Immutable stage-versioned analysis snapshots | In Review | Replay/diff/migration/stable-reference evidence |
| ADR-0009 | Bounded, human-adopted routing alternatives | In Review | route corpus, feasibility, ranking, explanation and adoption tests |
| ADR-0010 | Advisory snapshot-based capacity/opportunity analysis | In Review | shop data definitions, stale-data, delivery and accounting-separation tests |
| ADR-0011 | Seeded, versioned uncertainty representation | In Review | distribution policy, calibration, convergence and usability evidence |
| ADR-0012 | Revision comparison with revision-bound proposed PMI evidence | In Review | AP242/legacy corpus, translator fidelity, mapping ambiguity and human-confirmation tests |
| ADR-0013 | Governed correction-to-suggestion learning | In Review | data rights, cohort/bias/backtest/privacy and activation-control evidence |
| ADR-0014 | Vendor-neutral, read-only-first CAM reconciliation | In Review | supported adapter/version corpus, semantic mapping and security evidence |

Only a designated architecture/product/security review group may change an ADR to Accepted. Record date, approvers, evidence, and dissent in the ADR and here.
