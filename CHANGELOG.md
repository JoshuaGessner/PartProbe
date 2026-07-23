# Changelog

> **Status:** In Review  
> **Last updated:** 2026-07-23  
> **Related requirements:** All  
> **Related ADRs:** ADR-0001–ADR-0014  
> **Open questions:** None  
> **Dependencies:** Release governance  
> **Supersedes:** None

All notable project changes are recorded here. This project follows Keep a Changelog structure once releases begin.

## [Unreleased]

### Added

- Phase 0 repository governance and planning documentation baseline.
- Research, domain, requirements, architecture, UX, quality, delivery, and decision records.
- Initial fixture manifest and validation expectations.
- Planning metadata/link validator and fixture hash utility.
- Advanced planning for routing, capacity/economics, uncertainty, revision/PMI/coverage, feature explanation, learning, CAM, availability, sourcing, and bid priority.
- ADR-0009–ADR-0014, expanded traceability through REQ-F-065/DATA-042/TEST-099, staged roadmap, and governed human-adoption/versioning boundaries.
- Rust 1.94.1 workspace with domain, estimation-engine, and test-support crates.
- Typed units, fixed-precision money/currency, provenance/version wrappers, explicit value states, typed DAG validation, and canonical calculation snapshots.
- Executable TASK-001 tests, passing three-OS CI evidence, dependency record, and validation evidence.

### Security

- Established local-first, no-external-upload-by-default policy for technical and quote data.
