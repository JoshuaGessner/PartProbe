# Experience Principles

## Metadata

- **Status:** Draft
- **Last updated:** 2026-07-22
- **Related requirement IDs:** UX-001 through UX-010, REQ-F-021, REQ-F-039
- **Related architecture decision IDs:** ADR-0001
- **Open questions:** Which density mode should be the default for each estimator persona?
- **Dependencies:** [Design system](design-system.md), [interaction patterns](interaction-patterns.md)
- **Supersedes / superseded by:** None

## Principles

1. **Show the evidence behind every number.** A total is a navigable summary, never a dead end. Calculated, suggested, imported, historical, and overridden values have visible provenance.
2. **Keep a human in control.** Automation proposes and explains; the estimator accepts, modifies, or rejects it. Undo is immediate and an override asks for a reason only at the durable/audit boundary.
3. **Keep the part in context.** Model geometry, drawing-driven requirements, route, costs, and confidence must remain cross-linked; selection in one makes the related evidence visible in another.
4. **Earn density.** Favor compact, aligned information with generous legibility, progressive disclosure, resizable panels, saved views, and an optional comfortable mode—never tiny type or hidden state.
5. **Avoid destructive ambiguity.** Distinguish draft from approved, cost from price, estimate from actual, automatic from manual, and warning from blocking error by text, icon, color, and location.
6. **Respect expert flow.** Keyboard navigation, command palette, type-ahead search, batch editing, copy/paste, recent items, and sensible focus are baseline capabilities.
7. **Recover without punishment.** Import failures, missing drawing data, and uncertain geometry explain the next action and preserve work. A low-confidence result is useful context, not a silent failure.
8. **Protect sensitive work by default.** No surprise upload, telemetry, external preview, or data-sharing affordance. Export and print reveal scope and leave an audit event when required.

## Experience measures

Validate with usability testing: a trained estimator creates and revises a first-slice quote without a mouse-only step; can identify why a cost changed within 30 seconds; can restore an accidental routing edit; and can state current confidence reasons without opening a help manual.
