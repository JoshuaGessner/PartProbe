# Accessibility Requirements and Interaction Standard

## Metadata

- **Status:** Draft
- **Last updated:** 2026-08-16
- **Related requirement IDs:** UX-007 through UX-010, REQ-NF-004
- **Related architecture decision IDs:** ADR-0001
- **Open questions:** Confirm minimum supported assistive-technology/browser-engine combinations for each OS.
- **Dependencies:** [Design system](design-system.md), [interaction patterns](interaction-patterns.md), framework spike
- **Supersedes / superseded by:** None

## Commitment

The product targets WCAG 2.2 AA-oriented outcomes for the desktop UI where applicable and native platform accessibility APIs through the chosen webview. This is an engineering requirement and a test program, not a claim of formal certification. Use semantic HTML controls first, then tested ARIA patterns only when a native semantic equivalent is unavailable.

## Requirements

- Every function is operable without a pointer and without time-critical gestures. Keyboard focus is visible at 3:1 contrast or better and never obscured by sticky panels.
- Text, numeric values, field labels, units, and status explanations have at least 4.5:1 contrast at normal text size; non-text UI indicators meet 3:1. Use actual theme token tests, not visual judgment.
- Status never relies on color, shape, or model position alone. Confidence, selected geometry, errors, manual overrides, and validation include text plus icon/state.
- Screen readers receive named landmarks, headings, labels, row/column context, state, errors, and calculated-result updates. Announce non-urgent calculation completion politely; do not announce every transient cell update.
- Tables provide an accessible alternative for visual cost charts and a keyboard grid model where spreadsheet navigation is appropriate. Do not use ARIA grid merely for layout.
- Custom viewer content exposes a synchronized textual model tree, property list, and feature list. Spatial-only tasks include a non-visual method or are marked as requiring visual review.
- Respect OS scale, browser zoom, high-contrast/forced-colors when available, reduced motion, and user-selected theme/density. Do not lock text size or suppress focus outlines.
- Errors identify the field and describe correction in text; summaries link to each error and return focus predictably.

## Accessible composite controls

Command palette: dialog role, labelled search, result count, arrow navigation, active-result announcement, Escape closes/returns focus. Menus: button disclosure semantics, roving focus, Escape close. Inspector tabs: tablist/tab/tabpanel semantics. Toasts: nonessential only, pauseable where persistent; never the sole delivery of a failure. Split panes: keyboard-resizable or provide layout presets/reset.

## Test matrix and evidence

Run automated semantic/contrast checks in UI CI and manually test core workflows on: Windows with Narrator and Accessibility Insights; macOS with VoiceOver and Accessibility Inspector; Linux with Orca/AT-SPI on the supported distribution. Slint’s documentation is a useful reminder that custom widgets need explicit roles, labels, and actions ([Slint accessibility guidance](https://docs.slint.dev/latest/docs/slint/guide/development/best-practices/)); the same discipline applies to DOM controls. Capture test date, OS/webview version, flow, defects, and remediation in release evidence.

GUI-3 supplies a preliminary Apple-Silicon smoke. The rendered accessibility tree exposed the skip link, level-one/level-two headings, named application state, named native button, and source/estimate description lists; Tab produced visible focus, Return opened the picker, and Escape cancelled it. Reduced-motion and forced-color CSS fallbacks are present. The Linux package follow-up requires the exact visible host-owned `Select STEP model` window, the XDG portal's unique exact showing `File Chooser Widget`, and the exact live `Files` table before navigation. It sends GTK's documented `/` location-popup binding from that owning file-list widget, requires the resulting unique focused location entry, enters the complete absolute synthetic-fixture path in that native control, verifies the exact accessible text, and presses Return while the entry retains focus. Run 31971041371 proves the earlier flattened AT-SPI table-cell selection was not authoritative: although the prism cell reported selected, GTK accepted its automatically selected first row and PartProbe correctly exposed `Selected model source: cube_10mm.step`. The corrected gate avoids row-index inference and requires the unique exact showing application-owned prism label, picker closure, and an enabled Analyze control; any other source label fails immediately. Hosted confirmation remains pending. No path enters the WebView. No Orca/VoiceOver/Narrator session, automated contrast, zoom/scaling, high-DPI, complete estimate-form Linux/Windows workflow, or full acceptance pass has occurred, so TEST-012 and the non-negotiable examples remain open.

## Non-negotiable acceptance examples

An estimator can import a model, confirm units, select a feature, change an operation time, read the changed cost, record an override reason, and preview a quote using keyboard only. A screen-reader user can discover current quote/part, warning count, selected feature, confidence reasons, and field validation without interpreting the viewport. If a capability cannot meet that bar in the first slice, document the limitation and offer a supported equivalent path before claiming completion.
