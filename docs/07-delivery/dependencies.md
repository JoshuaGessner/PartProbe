# Dependency Policy and Candidates

> **Status:** In Review  
> **Last updated:** 2026-07-23  
> **Related requirements:** REQ-NF-001, REQ-NF-005, REQ-NF-010  
> **Related ADRs:** ADR-0001–ADR-0007  
> **Open questions:** Final versions, license compatibility, vendor terms  
> **Dependencies:** Spike evidence  
> **Supersedes:** None

TASK-001 added three exact, reversible Rust dependencies after the documented spike review: `rust_decimal 1.42.1`, `serde 1.0.229`, and `serde_json 1.0.151`. Their selected features, active transitive graph, licenses, build/runtime behavior, ownership, and removal paths are recorded in [the dependency record](dependency-record.md). All other families below remain candidates—not approvals:

| Candidate | Purpose | Maintenance/license/security review |
|---|---|---|
| Tauri 2 + Leptos | Desktop shell and CSR UI | ADR-0001 spike; WebView/runtime policy; verify licenses/releases |
| wgpu | Cross-platform 3D rendering | ADR-0003 version/backend/platform/adapter proof; direct/transitive/native-backend licenses, advisories/owner, build and package provenance |
| Open CASCADE Technology | Exact STEP/B-rep worker | ADR-0002/0005 license, ABI, packaging, CVE and sandbox review |
| Mesh/3MF/STL crates or libraries | Mesh intake/validation | Corpus quality, maintenance, format fidelity, license review |
| SQLite + Rust binding/ORM candidate | Local persistence | Bundled/system choice, migration API, unsafe surface, license review |
| rust_decimal | Currency arithmetic; added for TASK-001 only | ADR-0007 remains In Review; precision/overflow/Serde tests pass locally; exact dependency review is recorded separately |
| Serde + Serde JSON | Versioned DTO and canonical snapshot serialization; added for TASK-001 only | Compact JSON is a reversible internal spike contract, not yet an external interchange promise |
| Dimensional-units crate or project types | Unit safety | Ergonomics, serialization, maintenance, compile-time weight |
| UUID, hashing, time crates | Identity/integrity/time | Algorithm/features, OS entropy, parsing/serde surface |

Before addition, record exact version/source, purpose, alternatives, direct/transitive license, maintenance/release condition, unsafe/native/build scripts, network behavior, advisories, update owner, and removal cost. Generate lockfile/SBOM/license evidence at release gates.
