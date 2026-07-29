# Usability Test Plan

## Metadata

- **Status:** Draft
- **Last updated:** 2026-07-29
- **Related requirement IDs:** UX-001 through UX-012, TEST-012, TEST-014
- **Related architecture decision IDs:** ADR-0001
- **Open questions:** Recruiter source and representative controlled-data test fixtures; final measurable first-slice targets.
- **Dependencies:** [Screen inventory](screen-inventory.md), [accessibility](accessibility.md), representative fixtures, framework spike
- **Supersedes / superseded by:** None

## Objectives

Evaluate whether experienced estimators can form, understand, correct, and approve an auditable CAD-assisted estimate without losing model/drawing context. Test product behavior, not whether participants can decipher an unfinished prototype. Never use real customer-controlled CAD data without explicit authorization; use anonymized or synthetic fixtures.

## Participants

Recruit 8–12 participants across at least: 4 estimators (including 2 senior), 2 programmers/manufacturing engineers, 1 quality role, and 1 shop manager/purchasing role. Include 2 participants who primarily navigate with keyboard and, where possible, an assistive-technology user. Record experience, current estimating tools, CAD familiarity, OS, display scaling, and relevant accommodation needs—not unnecessary personal information.

## Method

Moderated 60–75 minute sessions: five-minute orientation, task scenarios, think-aloud where comfortable, brief debrief. Use an instrumented prototype/local test build, screen/audio capture only with consent, and observer notes. Do not coach task completion; offer help only after a participant declares a block. Separate observed task failure from a participant’s domain disagreement with an assumption.

## Scenarios and measures

| Scenario | Success evidence | Primary measures |
|---|---|---|
| Triage a new RFQ | Finds package gaps, creates a quote, assigns ownership | Completion, misclassification, time, confidence |
| Import a model with unit ambiguity | Confirms/corrects units and explains warning impact | Correct decision, error recovery, warning comprehension |
| Configure rates and pricing | Creates or dry-run imports a rate card, corrects an invalid row, resolves one overlap, and explains which rate/version an estimate selected | Completion, validation recovery, selection-trace comprehension, keyboard operation |
| Build first estimate | Selects material/stock/process, adjusts setup/runtime, adds inspection/risk | Completion, unnecessary steps, source comprehension |
| Explain changed price | Locates cost driver and override history | Time to explanation, accuracy, trust rating |
| Revise a model | Compares candidate analysis, preserves approved routing, records decision | Data-loss incidents, revision understanding |
| Prepare approval/customer output | Identifies blockers and produces correct preview | Approval error rate, export comprehension |
| Keyboard-only subset | Executes import-review-edit-cost sequence without pointer | Completion, focus loss, shortcut discoverability |
| Assistive-tech subset | Locates quote state, warning, selected feature, validation and cost change | Semantic/announcement defects, completion |

## Provisional acceptance targets

The first formative iteration targets at least 80% of representative participants completing the first five core scenarios without moderator rescue; it is a redesign trigger, not release acceptance. The vertical-slice release gate remains the 90% critical-task target in [success metrics](../00-product/success-metrics.md). At every stage, at least 90% must correctly distinguish estimated cost, risk allowance, markup, margin, and selling price and explain the selected rate/version; no participant may lose a durable edit or have an approved routing silently changed; keyboard-only users complete the subset; and all severity-1/2 accessibility defects are fixed before declaring the vertical slice usable. Time is diagnostic until baseline work is observed.

## Data collection and analysis

Capture task outcome, time, errors, backtracks, help requests, observed confusion, confidence (1–5), SEQ per task, and post-session SUS or a comparable standardized survey. Tag findings by role, scenario, UI surface, severity, and root cause: discoverability, terminology, visual hierarchy, interaction, calculation explanation, data/fixture quality, or accessibility. Prioritize by frequency × severity × workflow risk. A design change closes only after retest evidence or a documented rationale.

## Research ethics and security

Store recordings/notes locally in the approved research location with access limited to the research team; redact identifiers from synthesis. Do not send model files, recordings, or quote values to third-party transcription/AI tools by default. Consent forms explain purpose, capture, retention, and withdrawal process. Accessibility feedback is handled as product feedback, not a request for a participant to disclose a diagnosis.
