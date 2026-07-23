# Interaction Patterns

## Metadata

- **Status:** Draft
- **Last updated:** 2026-07-22
- **Related requirement IDs:** UX-002 through UX-010, REQ-F-021, REQ-F-031, REQ-F-039
- **Related architecture decision IDs:** ADR-0001
- **Open questions:** Exact keyboard shortcut map and conflict policy; threshold for requiring override rationale.
- **Dependencies:** [Design system](design-system.md), [accessibility](accessibility.md), [model review workflow](model-review-workflow.md)
- **Supersedes / superseded by:** None

## Core interactions

| Pattern | Required behavior |
|---|---|
| Inline edit | Enter/F2 activates; Escape restores pre-edit value; Enter commits and moves according to grid rules; units/format are visible; invalid value stays focused with text reason. |
| Override | Shows baseline, new value, source, and reason. A draft change may be briefly staged; durable calculation/routing override records user, time, reason, old/new value. |
| Recalculation | A non-blocking status names affected items; totals show `Updating` then timestamp/version. Never animate a changed money value without an explanation link. |
| Selection synchronization | Selecting geometry, feature, setup, tool, operation, cost line, or warning highlights related objects and provides a `why linked` explanation. Multi-select never silently implies causal linkage. |
| Undo / redo | Undo stack is workspace-local and names action; approved snapshots are immutable and require a new revision rather than undoing approval. |
| Command palette | `Ctrl/Cmd+K`; searches commands and permitted records; supports keyboard actions; shows shortcut and disabled reason. |
| Context menu | Keyboard `Shift+F10` and pointer invocation; has the same action availability as adjacent menu/toolbars; no unique critical action. |
| Drag/drop | Import cues show accepted formats and local-only handling. Dropped files enter validation, not immediate analysis; a reject offers reason and remediation. |
| Long-running job | Import/analyze/report tasks show stage, elapsed time, cancelability, and background continuation. Results never replace approved work automatically. |
| Confirmation | Use only for irreversible/destructive actions, approval, sending/exporting controlled data, and rule-changing changes. State exact effect and default to safe action. |

## Keyboard navigation baseline

`Tab` moves through document controls in visual order; `Shift+Tab` reverses. Arrow keys navigate a focused grid/list/tree; `Space` changes selection where appropriate; `Enter` opens or edits; `Ctrl/Cmd+F` focuses contextual search; `Ctrl/Cmd+S` saves draft; `Ctrl/Cmd+Z` / `Shift+Ctrl/Cmd+Z` undo/redo; `?` opens shortcut help only when no editable field owns it. Shortcuts must not replace visible commands.

## Tables and batch work

Grid selection has a visible active cell and announced row/column/value. Header menus expose sort, filter, hide, resize, and reset. Pasting previews validation failures before durable commit; a batch override uses one rationale and produces per-field audit entries. Filtering never deletes rows; saved filters are named and recoverable.

## Error recovery

Every error explains *what happened*, *what remains safe*, and *what to do next*. A CAD import failure retains file metadata/hash when safely available and lets the user retry with unit confirmation or attach a different revision. Parser diagnostics contain an ID and safe metadata, never copied model content. Focus moves to the summary only when requested; otherwise it stays at the field/action that caused the failure.
