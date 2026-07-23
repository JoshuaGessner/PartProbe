# ADR-0008 — Model Analysis Versioning

> **Status:** In Review  
> **Last updated:** 2026-07-22  
> **Related requirements:** REQ-F-002–REQ-F-007, REQ-F-010; REQ-NF-009; GEO-001–GEO-015  
> **Related ADRs:** ADR-0002–ADR-0005, ADR-0007  
> **Open questions:** Snapshot retention and stable topology mapping policy  
> **Dependencies:** Geometry spike  
> **Supersedes:** None

## Decision proposed

Version the whole analysis pipeline and each stage independently. Persist source hash, format/reader/kernel versions, unit/scale decision, healing policy/actions, stage configuration, result schema version, timings, warnings, confidence reasons, and approved snapshot.

## Rules

- A different source hash or material configuration creates a new analysis identity.
- Re-running a newer algorithm creates a sibling snapshot and a diff proposal.
- Feature/setup/runtime consumers pin exact snapshot IDs.
- Human review/overrides reference original geometry/result IDs and are never overwritten.
- Schema migrations preserve original payloads or a verifiable transformed representation.

## Acceptance gate

Same-input determinism within documented tolerances, readable legacy fixture snapshots, explicit cross-version diff, stable mapping behavior documented, and proof that re-analysis cannot mutate approved routing.
