# PartProbe Design System

## Metadata

- **Status:** Draft
- **Last updated:** 2026-08-09
- **Related requirement IDs:** UX-001 through UX-010, REQ-NF-004
- **Related architecture decision IDs:** ADR-0001
- **Open questions:** Validate contrast tokens on calibrated shop-floor displays; choose icon license and font distribution strategy.
- **Dependencies:** [Accessibility](accessibility.md), [interaction patterns](interaction-patterns.md)
- **Supersedes / superseded by:** None

## Visual direction

PartProbe is a precise engineering workstation: quiet graphite surfaces, sharp data alignment, restrained blue interaction accents, and material/process signal colors. It avoids simulated metal, faux-CAD skeuomorphism, gradients as decoration, and generic enterprise card grids. A model viewport is visually distinct but never a black box: its legend, selection state, and analysis status are part of the system.

## Foundations

| Token family | Definition |
|---|---|
| Typeface | Inter or approved bundled sans for UI; tabular numerals for quantities/currency; 11/13/14/16/20/28 px scale. Never use a non-bundled remote font. |
| Spacing | 4 px base: 4, 8, 12, 16, 20, 24, 32. Table row height 28 compact / 36 comfortable. |
| Radius | 2 px input/menu; 4 px panel; 6 px dialog. No pill controls except compact status chips. |
| Elevation | Borders first; 1 px shadows only for floating menus/dialogs; no persistent card shadows. |
| Layout | 8 px shell grid; resizable panes with minimum readable widths; 44 px workspace header; 28 px table density default. |
| Motion | 120–180 ms opacity/position transitions; no motion for calculation updates; respect reduced-motion preference. |

## Semantic color tokens

Use named semantic tokens, not raw colors in components. Dark is default for long engineering sessions; light is equally complete. Initial candidate tokens (final values must pass contrast testing): `surface-canvas`, `surface-panel`, `surface-raised`, `border-subtle`, `text-primary`, `text-secondary`, `text-muted`, `action-primary`, `action-focus`, `selection`, `success`, `warning`, `danger`, `info`, and confidence `high/medium/low/unknown`.

Status always combines color with a word/icon: `High confidence`, `Review required`, `Blocking error`, `Manual override`, or `Outdated source`. Do not encode process, feature, or cost category only by hue.

## Components and behavior

| Component | Rules |
|---|---|
| App frame | Native titlebar by default, global navigation, command palette, persistent connection/save indicator. |
| Panels | Clear title/metadata; collapse only secondary information; resize and reset layout available. |
| Buttons | One primary action per local context; destructive actions use text and confirmation when irreversible. |
| Fields | Visible label, unit/format suffix, validation message tied to field, source/override marker where applicable. |
| Tables | Stable row IDs, virtualized rows, pinned identity column, sortable headers, column chooser, density modes, keyboard grid behavior. |
| Inspector | Contextual right pane shows selected object evidence, editable properties, history, and related objects. |
| Status chips | Concise, not a replacement for detailed explanation; click opens reasons. |
| Menus | Contextually anchored, keyboard navigable, command-label + shortcut, no icon-only critical action. |
| Empty/loading/error | State what is missing/working/failed, show safe next action and diagnostic ID; never a blank viewport. |

## Data visualization

Show money in fixed-precision formatted currency and units beside every dimension. Cost breakdown uses a sorted bar/list with actual numeric labels and a readable table alternative. Quantity comparison uses aligned scenario columns, not a decorative chart. Model legends synchronize with feature, setup, and routing selection. Never infer precision from a long decimal display.

## Themes and density

Dark and light themes map every semantic token; neither is a color inversion. Compact is default for expert workstation use; comfortable increases row/panel spacing, not just text zoom. OS text scaling and browser zoom remain supported. Theme and density are user preferences, and a quote never relies on a personal preference to communicate meaning.

## Current GUI-3 implementation boundary

GUI-3 establishes only the shell foundation: project-owned CSS variables, quiet graphite surfaces, restrained blue focus/action treatment, sharp borders, native window chrome, semantic status text, tabular numerals, a skip link, visible focus, reduced-motion handling, and forced-color fallbacks. It intentionally uses no remote font or default-looking widget library. Light theme, density switching, reusable component crates, measured token contrast, resizable panes, tables, inspector, menus, and viewport styling remain design-system work and are not implied complete by this first screen.
