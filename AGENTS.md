# PartProbe Agent Instructions

> **Status:** In Review  
> **Last updated:** 2026-08-22
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
53. The developer STEP runtime root and external worker workspace must be explicit deployment configuration. Derive the worker and native-library paths only from a successfully verified runtime manifest. Missing, invalid, changed, incomplete, unmanifested, or wrong-host runtime state remains visibly `Unavailable`; do not fall back to independent worker/library paths, in-process parsing, ambient discovery, numeric defaults, or a falsely successful result.
54. GUI-4 contract v3 exposes exactly five application commands: contract discovery, source selection, analysis, analysis cancellation, and draft-estimate evaluation. Keep every command in the shared contract, Tauri manifest, exact permission, window capability, and host regression evidence.
55. Analysis cancellation is selection-token-bound and cooperative, with the supervisor's existing bounded forced cleanup as fallback. A new source may request cancellation, but it must not silently replace an active native session or present cleanup as complete before the worker path returns.
56. GUI-4 rates and pricing are explicit, confirmed, session-only developer inputs. Build typed rate governance and pricing-policy values in the native adapter, invoke only `DraftEstimateSession::evaluate`, and never present those ephemeral approvals as a durable shop library, production default, approved quote, or saved record.
57. Canonical-unit and warning-review confirmations are bound to the current analysis revision. Clear them whenever source or analysis state changes; never carry a review decision into re-analysis. Present user-requested cancellation distinctly from worker failure and keep retry state explicit.
58. The GUI-5 real-worker smoke is an opt-in native test requiring an explicit verified runtime root and external worker workspace. Keep it ignored in ordinary CI unless that job constructs the exact pinned native runtime; never promote feature-off or synthetic-port results into configured-native evidence.
59. Developer native runtimes must be assembled only from explicit pinned inputs into a new output, include the complete target OCCT shared-library closure, verify relative-path manifests and every copied artifact during assembly and again inside the desktop host before launch, reject overwrite/ambient discovery/unmanifested files, and remain labeled internal until legal, signing, containment, and three-OS package evidence pass.
60. Treat worker workspace-output monitoring as poll-bounded supervisor containment, not a hard filesystem quota or sandbox. Preserve the staged-input size/read-only invariant, bound aggregate non-input bytes and entry count during execution and after exit, reject special or nested entries, terminate on violation, and recursively clean only the supervisor-owned private job directory.
61. On supported Linux `x86_64`/`aarch64` workers, prepare irreversible `no_new_privs` before creating the cancellation reader and install the thread-synchronized parser seccomp filter only after the authorized asset has been claimed into immutable bytes. Keep socket creation and process/thread/exec creation denied, fail closed with a sanitized containment diagnostic, and do not generalize that Linux-only syscall evidence into filesystem sandbox, native-OCCT, or three-OS containment claims.
62. Native Linux OCCT evidence must construct the exact pinned source in an opt-in, non-distributing job; assemble and re-verify a new manifest-bound runtime; prove every `libTK*` dependency resolves inside that runtime; and run the configured desktop-host smoke under the production parser filter. Do not cache or publish those developer binaries, and do not promote a headless Ubuntu smoke into signed packaging, windowed-GUI, Windows, legal, or supported-importer evidence.
63. Native Windows x64 OCCT evidence must pin the Visual Studio generator and `x64` generator platform, fingerprint installed runtime DLLs rather than build-time import libraries, keep the complete `TK*.dll` closure app-local beside the worker, verify every worker/DLL PE machine and OCCT import with an explicit reviewed dependency tool, and run the configured desktop-host smoke. Invoke the dependency tool's header and dependent modes separately so each documented output grammar is validated without mixed-mode ambiguity. Do not infer bundled MSVC runtime, signing, installer, windowed-GUI, legal, or supported-importer evidence from that developer job.
64. The source repository is public. Commit only reviewed public synthetic fixtures or assets with documented redistribution rights. Never commit customer/shop/private CAD, drawings, estimates, rates, credentials, signing material, controlled technical data, or private validation artifacts; keep private validation outside the repository and public CI.
65. Linux desktop package evidence must use an explicit supported baseline, pinned build tools, reviewed Tauri prerequisites, bundled local frontend content, package inspection, private ephemeral extraction, and a bounded launch of the exact package-contained executable under a virtual display. Create the virtual display before starting the session bus or desktop-portal services, and provide an explicit window manager for interactive evidence. Keep the workflow opt-in and non-publishing; reject package formats whose build path fetches unpinned helper tools until exact sources/hashes are governed. An unsigned extracted-payload survival smoke alone is not package installation/registration, signed distribution, native-OCCT integration, installer acceptance, accessibility acceptance, or production support.
66. A packaged desktop host may select a native runtime only from the fixed `partprobe-native-runtime` child of Tauri's resolved resource directory, unless an explicit developer `PARTPROBE_NATIVE_RUNTIME` override is present. Re-verify that root before deriving worker/library paths; never search adjacent directories, accept independent launch paths, or silently fall back after verification failure.
67. Before a Linux package copies a verified native runtime, create a separate new package runtime that materializes only already-validated local library aliases, rewrites those manifest entries as exact regular-file size/hash evidence, and re-verifies the complete output. Do not weaken post-extraction verification to accommodate package-tool symlink conversion, and do not mutate the original assembled runtime.
68. Interactive desktop evidence must drive the exact packaged executable through its real keyboard/accessibility and host-owned dialog surfaces. Do not add test-only source-path bypasses, WebView filesystem authority, coordinate-only clicks, or synthetic command invocation and present them as native UI acceptance.
69. Accessibility-driven evidence must find exact semantic targets, require the intended live/showing/enabled state, use their selection/action interfaces or keyboard behavior, and confirm the resulting application state. Do not assume cross-process accessibility ownership or descendant topology when the platform bridge may flatten or reparent nodes; require a unique exact live match instead. A bounded sequence of documented user-equivalent activation methods may recover from a platform bridge that accepts an action without emitting the native response, but stop immediately after the expected application transition and log the method that worked. An accepted focus, selection, or action call is not evidence by itself; reject substring ambiguity, duplicate live controls, hidden controls, unrealized-widget fallbacks, coordinate targeting, and success inferred without the expected state transition.
70. Linux desktop builds must use the reviewed `tauri-plugin-dialog` XDG portal backend with its default GTK3 backend disabled. The supported Linux environment must provide the pinned portal implementation and session bus; do not silently fall back to an in-process chooser or claim portal evidence from service activation alone. Give host-owned native dialogs a deliberate stable title, and bind keyboard evidence to the exact visible platform window before sending input; AT-SPI focus on a child alone is not proof of X11 keyboard focus. Send chooser key bindings only from the documented owning widget, and require the expected unique live control plus exact entered text before continuing; successful key injection is not evidence that a shortcut was handled. Do not infer a GTK file-row selection from a flattened AT-SPI cell or an ancestor child index: GTK may retain its automatically selected first row. For exact-fixture evidence, enter the complete authorized source path in the focused native location entry, verify its exact accessible text, and let the chooser resolve and accept that file.
71. Accessibility evidence must distinguish visible DOM text, platform Text-interface content, and an explicit accessible name or description. When a cross-platform acceptance gate needs exact application state, publish one deliberate path-free application-owned accessible label and verify that exact showing label; do not infer bridge behavior from visual text, depend on undocumented Text-interface projection, or weaken state evidence to dialog closure alone. If any other application-owned source label appears while waiting for the governed fixture, fail immediately rather than accepting closure or a generic selected state.
72. Tests for helpers that accept an absolute monotonic deadline must control the monotonic clock and pass a demonstrably later deadline. Never use a small numeric literal as an absolute deadline because runner uptime can make it already expired.
73. Cross-platform tests must construct absolute paths with the current host's path semantics unless they are explicitly testing a named foreign path grammar. A POSIX leading slash is not a Windows absolute path.
74. When a live accessibility tree can change during polling, classify expected, wrong, and ambiguous state from one traversal snapshot per iteration. Do not make a fail-fast decision by comparing separate traversals whose state may differ.
75. The TASK-004 ASCII/binary STL analyzers report unresolved source-coordinate evidence only. Do not infer physical units, expose closed volume/centroid for open or inconsistently wound meshes, accept unsupported binary attribute payloads silently, or promote exact fixture-edge matching into a production welding/tolerance policy.

## Documentation protocol

- Use stable IDs defined in `docs/INDEX.md`.
- Put canonical rules in one document and link to them elsewhere.
- Add the standard metadata block to important documents.
- Requirements move to Complete only with linked validation evidence.
- ADR status is `Proposed`, `In Review`, `Accepted`, `Superseded`, or `Rejected`; only authorized reviewers accept decisions.
- Record unresolved shop facts as assumptions or open questions, not as truth.

## Session closeout

Before ending meaningful work: run applicable checks, update project state, add a dated progress-log entry, update requirement/test links, and list any new risks or decisions.
