# PartProbe

> **Status:** In Review
> **Last updated:** 2026-07-29
> **Related requirements:** REQ-F-001–REQ-F-065; REQ-NF-001–REQ-NF-022
> **Related ADRs:** ADR-0001–ADR-0014
> **Open questions:** OQ-001–OQ-050
> **Dependencies:** Planning review and technical spikes
> **Supersedes:** None

PartProbe is a planned local-first, cross-platform CAD-assisted estimator for specialty machine shops. It will import engineering models, extract explainable geometry facts, propose editable manufacturing assumptions, and produce traceable cost and price foundations. It is an estimating assistant—not production CAM and not an autonomous quoting system.

## Current status

The project is in **Phase 0 — Discovery and Validation**. TASK-001 provides a reversible Rust calculation foundation and three-OS TEST-001 evidence. TASK-002 now provides locally passing empty/user-owned rate-card, deterministic resolution, pricing/rounding, synthetic-golden, and replay mechanics; cross-platform TASK-002 evidence remains pending. Broad production development is intentionally deferred until the documented readiness gates are reviewed.

Start with:

- [Documentation index](docs/INDEX.md)
- [Current project state](docs/PROJECT_STATE.md)
- [Agent rules](AGENTS.md)
- [Planning prompt](machine_shop_estimator_prompts/01_project_setup_and_deep_planning_prompt.md)

## Security posture

CAD models, drawings, quotes, and shop data are treated as potentially sensitive. No project workflow may upload them to telemetry, analytics, AI, or other external services by default. The project does not claim AS9100, CMMC, ITAR, NIST, or export-control compliance.

## Development

The calculation workspace is scaffolded, but no desktop application, geometry engine, persistence layer, or production estimator exists. TASK-001 is complete with passing Windows, Linux, and macOS CI evidence; TASK-002 implementation passes locally and awaits three-OS evidence before closure. The next planned development slice after that is TASK-003. Commands and current limits are documented in [PROJECT_STATE.md](docs/PROJECT_STATE.md).
