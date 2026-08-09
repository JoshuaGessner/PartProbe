# PartProbe Agent Instructions

> **Status:** In Review  
> **Last updated:** 2026-08-09
> **Related requirements:** All  
> **Related ADRs:** ADR-0001–ADR-0014  
> **Open questions:** None  
> **Dependencies:** `docs/INDEX.md`, `docs/PROJECT_STATE.md`  
> **Supersedes:** None

These rules are mandatory for every person or agent changing this repository.

1. Read `docs/INDEX.md` and `docs/PROJECT_STATE.md` before changing code.
2. Read relevant requirements, architecture, UX, geometry, and calculation documents before modifying a module.
3. Do not contradict an approved document without creating or updating an ADR.
4. Do not silently change calculation behavior.
5. Do not silently change geometry interpretation.
6. Every calculation change requires a documented rule, tests, a versioning or migration decision, and updated worked examples.
7. Every geometry-analysis change requires documented behavior, model fixtures, expected measurements, regression tests, and confidence behavior.
8. Every feature-recognition change requires test parts, expected detections, known false positives and false negatives, and confidence criteria.
9. Every data-schema change requires a migration, backward-compatibility consideration, and updated documentation.
10. Every UI feature must use the project design system.
11. Do not introduce default-looking widgets without deliberate styling.
12. Do not add dependencies without documenting the reason, maintenance condition, licensing, and security implications.
13. Avoid platform-specific behavior unless isolated behind an interface.
14. Keep Windows, Linux, and macOS support intact.
15. Keep the application runnable at the end of each implementation session.
16. Update `docs/PROJECT_STATE.md`, `docs/07-delivery/progress-log.md`, and affected requirement statuses after meaningful work.
17. Do not claim a feature is complete unless its acceptance criteria pass.
18. Do not describe the product as AS9100, CMMC, ITAR, NIST, or export-control compliant without a formal external determination.
19. Do not send CAD files, drawings, models, specifications, prices, or estimate data to external services by default.
20. Do not add AI functionality that transmits controlled files externally without an explicit ADR and deployment policy.
21. Prefer deterministic and explainable calculations.
22. Manual overrides must record original value, new value, user, time, and reason.
23. Preserve an audit trail for approved quotes, model revisions, and rate changes.
24. Do not treat mesh-derived feature recognition as equivalent to solid-model recognition.
25. Do not imply rough runtime estimates are production CAM simulations.
26. Preserve the source model and a cryptographic hash of the analyzed file.
27. Record model units, scale decisions, healing actions, and import warnings.
28. Never automatically overwrite a human-approved routing after re-analysis.
29. Preserve the accepted deterministic estimate as the baseline for every advanced analysis; recommendations, ranges, readiness adjustments, and opportunity-cost views must remain distinguishable layers.
30. Do not present a route, bid decision, learned rule, sourcing choice, requirement interpretation, or revision-cost allocation as authoritative without human review and recorded adoption.
31. Never collapse missing, stale, inapplicable, blocked, or uncertain data into zero or an apparently valid value.
32. Keep theoretical capability, current availability/readiness, reservation state, and scheduling commitments distinct.
33. Keep accounting cost, contribution, opportunity cost, selling price, and external benchmark concepts distinctly typed and labeled.
34. Estimator corrections, CAM comparisons, and actuals may produce governed evidence and rule suggestions; they must not silently mutate active calculation rules or historical estimates.
35. Revision comparison must preserve topology ambiguity, tolerance/mapping evidence, and the estimate/rate basis used for every displayed cost delta.
36. PMI-derived requirements are proposed, revision-bound evidence until confirmed; graphical PMI is not semantic PMI.
37. Integration adapters must expose a vendor-neutral core contract, declared source/version, bounded payload, and controlled-data policy.
38. Published quotes and historical analyses pin route, requirement, source, library, algorithm, capacity, uncertainty, and policy versions needed for replay; later releases never silently recalculate them.
39. Treat `docs/PROJECT_STATE.md` and linked validation evidence as the authority for present-tense implementation and test status; do not promote historical checkpoint counts or CI runs into current claims.
40. Distinguish a technical spike, internal developer test slice, developer alpha, supported product capability, and release acceptance. One-platform, optional-feature, synthetic-fixture, or session-only evidence does not establish production support.
41. Desktop code must use typed application services for authorization, CAD analysis, calculation, and persistence. The UI must not parse CAD, execute authoritative estimate rules, bypass audit/policy boundaries, or convert missing/conflicting inputs to zero.
42. Describe the geometry worker as partially OS-contained, not sandboxed, until target-specific network denial, filesystem confinement, resource enforcement, and descendant controls have approved evidence on every supported target.
43. A session-only developer GUI must visibly label provisional analysis and ephemeral state and cannot satisfy save/reopen, persistence, importer-support, milestone, or release acceptance criteria.
44. Desktop adapters must consume `DraftEstimateApplication` and `DraftEstimateResult` for the GUI-2 path; do not duplicate estimate formulas, rate selection, missing-state handling, or pricing logic in commands, view models, or components.
45. A GUI-2 draft result is available only after explicit unit/warning review, complete manual inputs, a pinned rate-card/effective-date/scope context, and a pinned pricing policy. Preserve `Unavailable` and `Blocked` states through every adapter.
46. `crates/desktop-contract` is the source of truth for desktop command names, event names, and pathless bridge DTOs. The Leptos frontend and Tauri host must consume it rather than duplicate strings or schemas.
47. Native file dialogs and source paths are host-owned. Never send a raw path, directory name, file bytes, CAD content, or geometry content into the webview; use a session-bound opaque selection ID and the minimum reviewed display metadata.
48. Every registered Tauri application command must appear in `AppManifest::commands`, an exact application permission, the intended window capability, and a host-contract regression test. Do not grant `default` permission sets when narrower permissions exist.
49. The production desktop baseline uses bundled local content, a restrictive CSP, native window decorations, and no frontend shell, HTTP, filesystem, opener, updater, upload, or dialog permission. Development-only localhost serving does not authorize a production network capability.
50. Keep the desktop host responsive while native dialogs and application jobs are active. Do not call blocking dialog APIs from a Tauri command; cancellation must preserve the last accepted session state unless the user explicitly clears it.
51. For GUI-4 intake, derive the source hash only after application-service authorization and audit, from the same already-open bounded grant later consumed by the worker. The desktop host must not pre-read CAD to manufacture a trusted request hash.
52. GUI-4 desktop analysis commands accept only opaque native session tokens. Invoke `DraftEstimateApplication` off the UI thread, retain `DraftEstimateSession` in native session state, and expose only path-free provisional DTOs; never send a source path, CAD bytes, or application/domain authority into the WebView.
53. The developer STEP worker, workspace, and native-library paths must be explicit deployment configuration. Missing or invalid configuration remains visibly `Unavailable`; do not fall back to in-process parsing, ambient executable discovery, numeric defaults, or a falsely successful result.
54. GUI-4 contract v3 exposes exactly five application commands: contract discovery, source selection, analysis, analysis cancellation, and draft-estimate evaluation. Keep every command in the shared contract, Tauri manifest, exact permission, window capability, and host regression evidence.
55. Analysis cancellation is selection-token-bound and cooperative, with the supervisor's existing bounded forced cleanup as fallback. A new source may request cancellation, but it must not silently replace an active native session or present cleanup as complete before the worker path returns.
56. GUI-4 rates and pricing are explicit, confirmed, session-only developer inputs. Build typed rate governance and pricing-policy values in the native adapter, invoke only `DraftEstimateSession::evaluate`, and never present those ephemeral approvals as a durable shop library, production default, approved quote, or saved record.

## Documentation protocol

- Use stable IDs defined in `docs/INDEX.md`.
- Put canonical rules in one document and link to them elsewhere.
- Add the standard metadata block to important documents.
- Requirements move to Complete only with linked validation evidence.
- ADR status is `Proposed`, `In Review`, `Accepted`, `Superseded`, or `Rejected`; only authorized reviewers accept decisions.
- Record unresolved shop facts as assumptions or open questions, not as truth.

## Session closeout

Before ending meaningful work: run applicable checks, update project state, add a dated progress-log entry, update requirement/test links, and list any new risks or decisions.
