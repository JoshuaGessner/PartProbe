# Prompt 2 — Begin Development From the Approved CAD-Assisted Estimation Plan

You are the senior Rust engineer responsible for beginning production development of the specialty machine-shop estimating application described in this repository.

The product is a cross-platform CAD-assisted desktop estimator. It imports 3D models, extracts geometric information, proposes machining assumptions, estimates runtime and supporting costs, and presents a transparent, editable quote foundation.

You must follow the approved planning documentation closely. Do not treat this prompt as permission to replace the established plan with your own unrelated architecture.

---

## 1. Mandatory Orientation

Before writing code:

1. Read `AGENTS.md`.
2. Read `docs/INDEX.md`.
3. Read `docs/PROJECT_STATE.md`.
4. Read the active milestone in `docs/07-delivery/milestones.md`.
5. Read the requirements linked to the next task.
6. Read relevant architecture decisions.
7. Read relevant UX documentation.
8. Read geometry, feature-recognition, runtime, and calculation rules for the modules being changed.
9. Inspect existing code and test status.
10. Check for uncommitted or incomplete work before restructuring anything.

Add a concise entry to the session progress log containing:

- Current phase
- Current milestone
- Selected task
- Related requirement IDs
- Related ADRs
- Acceptance criteria
- Expected files to change
- Expected model fixtures to use
- Known risks

Do not merely summarize the documents. Use them to guide implementation.

---

## 2. Core Technical Constraints

The application must:

- Run on Windows, Linux, and macOS
- Be written primarily in Rust
- Maintain platform-independent domain, geometry, runtime, and estimation engines
- Use the approved desktop and UI architecture
- Use the approved geometry kernel and rendering architecture
- Use fixed-precision decimal arithmetic for authoritative money calculations
- Use explicit units
- Use deterministic and explainable calculations
- Support versioned geometry, feature, runtime, feeds-and-speeds, and rate-card behavior
- Preserve manual override history
- Remain local-first unless the approved plan says otherwise
- Avoid external telemetry and external file transmission by default
- Use least-privilege access to files and operating-system capabilities
- Maintain a polished, custom, high-end interface
- Avoid default-looking widgets when the design system provides an alternative
- Preserve expected platform behavior and accessibility
- Keep the application runnable throughout development

The software must not claim to be production CAM and must not generate production G-code during the initial product phases.

If the approved ADR selects Tauri 2 and Leptos, implement the UI in Rust using Leptos and compile it for the Tauri desktop shell. Do not introduce a parallel JavaScript or TypeScript frontend.

If the approved ADR selects another Rust UI framework, follow that ADR.

If the approved geometry architecture uses native libraries through FFI, isolate unsafe code, document ownership and lifetime assumptions, and keep the public Rust interface safe.

---

## 3. Working Method

Work milestone by milestone and task by task.

For each task:

1. Confirm requirement IDs.
2. Confirm acceptance criteria.
3. Identify affected modules.
4. Identify model fixtures and expected outputs.
5. Add or update tests.
6. Implement the smallest coherent change.
7. Run formatting, linting, tests, and geometry regression tests.
8. Launch the application when UI behavior changes.
9. Update documentation.
10. Update requirement status.
11. Update `docs/PROJECT_STATE.md`.
12. Add a progress-log entry.
13. Report incomplete or deferred items honestly.

Do not perform broad rewrites without an ADR or explicit task.

Do not leave the repository in a partially migrated state.

---

## 4. Initial Development Objective

Unless the approved project state specifies a later task, begin by establishing the technical foundation and then implement the first CAD-assisted end-to-end vertical slice.

The first usable vertical slice should allow an estimator to:

1. Launch the desktop application.
2. Create an RFQ.
3. Select or create a customer.
4. Add one part.
5. Record part number, revision, and description.
6. Import a STEP, STL, or 3MF model.
7. Confirm model units.
8. View the model.
9. See import and geometry warnings.
10. See bounding dimensions, volume, and surface area.
11. Select material and density.
12. Calculate part mass.
13. Receive a rectangular or round stock suggestion.
14. See removed material volume.
15. Choose a broad process class.
16. Receive a rough setup-count suggestion.
17. Receive a coarse runtime estimate.
18. Edit machine, setup, stock, feed, speed, and runtime assumptions.
19. Add programming, inspection, tooling, outside-process, and risk values.
20. Enter one or more requested quantities.
21. Calculate internal cost.
22. Apply markup or target-margin pricing.
23. Display quantity-break pricing.
24. Display a transparent calculation breakdown.
25. Record assumptions.
26. Override an estimate value with a required reason.
27. Save the estimate.
28. Close and reopen it.
29. Preserve model hash and analysis versions.
30. Preview an internal estimate summary.
31. Preview a customer-facing quotation.
32. Build on Windows, Linux, and macOS.

The first vertical slice does not require full automatic feature recognition. It does require real CAD-assisted geometry extraction and a rough model-informed runtime estimate.

---

## 5. Recommended Foundation Order

Follow the approved roadmap. When consistent with it, use this sequence.

---

## 5.1 Workspace and Tooling

Establish or verify a Cargo workspace.

Recommended logical separation:

```text
apps/
└── estimator-desktop/

crates/
├── domain/
├── geometry-core/
├── geometry-import/
├── feature-recognition/
├── setup-planner/
├── tooling/
├── feeds-speeds/
├── runtime-estimation/
├── estimation-engine/
├── application/
├── persistence/
├── document-storage/
├── security/
├── import-export/
├── reporting/
├── platform/
├── model-viewer/
├── ui-components/
├── test-support/
└── sample-data/
```

Not every crate must be created immediately. Create a crate only when its boundary is useful.

Configure:

- `rustfmt`
- Clippy
- Unit tests
- Integration tests
- Geometry golden tests
- Cross-platform CI
- Dependency auditing
- License review
- Reproducible build instructions
- Development fixtures
- Structured logging with sensitive-data redaction

Document platform-native dependencies and packaging steps.

---

## 5.2 Domain Primitives

Implement and test foundational types before building many screens.

Potential types include:

- `Money`
- `Currency`
- `Rate`
- `Quantity`
- `Percentage`
- `Margin`
- `Markup`
- `Mass`
- `Length`
- `Area`
- `Volume`
- `Duration`
- `Angle`
- `UnitSystem`
- `CoordinateSystem`
- `PartNumber`
- `Revision`
- `QuoteId`
- `EstimateId`
- `CustomerId`
- `ModelId`
- `AnalysisId`
- `FeatureId`
- `SetupId`
- `ToolId`
- `MaterialId`
- `WorkcenterId`
- `OperationId`
- `RateCardVersion`
- `FeedsSpeedsVersion`
- `GeometryVersion`
- `FeatureRecognitionVersion`
- `RuntimeEstimationVersion`
- `CalculationVersion`

Use validated newtypes where they prevent invalid state.

Avoid unlabelled `f64` values moving between the geometry, runtime, and estimation engines.

Floating-point geometry math is expected where required, but authoritative currency must use fixed-precision decimals.

Document tolerance and rounding policies separately for:

- Geometry comparisons
- Unit conversion
- Runtime
- Currency
- Display formatting

---

## 5.3 Model File Intake

Implement a controlled model-ingestion service.

It must:

- Accept only approved formats
- Identify the actual file type
- Reject unsupported or suspicious inputs safely
- Record original filename
- Record file size
- Compute a cryptographic hash
- Record import timestamp
- Preserve the source file according to storage policy
- Detect or extract units when possible
- Require user confirmation when units are uncertain
- Record parser warnings
- Record healing actions
- Preserve importer version

Do not trust file extensions alone.

Do not log model content.

Use a temporary-file strategy that respects restricted-data deployment requirements.

---

## 5.4 Geometry Kernel Abstraction

Implement the approved geometry engine behind a safe Rust interface.

The public interface should not expose unnecessary native-library details.

A candidate abstraction should support:

- Importing exact solids
- Importing meshes
- Enumerating bodies
- Validating geometry
- Computing bounds
- Computing volume
- Computing surface area
- Computing center of mass
- Generating render meshes
- Querying faces, edges, and topology where available
- Geometry healing where supported
- Exporting diagnostics
- Cancellation
- Timeouts or containment where appropriate

If parsing or geometry operations can crash the process, follow the approved ADR for worker-process isolation.

Never let UI components call the geometry kernel directly. Use application services.

---

## 5.5 Geometry Analysis

Implement the initial geometry-analysis pipeline.

For each imported model, produce:

- Format
- Unit source
- Confirmed unit
- Scale
- Body count
- Shell count where available
- Solid-validity state
- Mesh watertightness when applicable
- Axis-aligned bounding box
- Oriented bounding box if in scope
- Volume
- Surface area
- Center of mass if supported
- Geometry warnings
- Analysis duration
- Geometry-engine version
- Confidence

Measurements must carry units.

Invalid or partially valid files must produce structured warnings rather than unexplained failure.

Do not continue to authoritative material and runtime calculations when scale is unresolved.

---

## 5.6 Model Viewer

Implement the approved 3D model viewer.

The first viewer should support:

- Fit to view
- Orbit
- Pan
- Zoom
- Orthographic and perspective modes if planned
- Standard views
- Bounding-box overlay
- Axis indicator
- Selection
- Basic body visibility
- Warning banner
- Unit display
- Model metadata
- Theme integration
- High-DPI rendering
- Keyboard shortcuts
- Accessible non-visual summaries of model properties

The viewer must fit the project design system.

Do not allow rendering details to leak into the domain or estimation engine.

---

## 5.7 Stock Suggestion

Implement the first stock-selection algorithm.

For the initial milestone, support:

- Rectangular stock
- Round stock
- User-specified stock

Inputs may include:

- Model bounding dimensions
- Candidate orientation
- Material
- Saw allowance
- Facing allowance
- Workholding allowance
- Standard-size list
- User preference

Outputs must include:

- Stock form
- Stock dimensions
- Stock volume
- Part volume
- Removed volume
- Yield
- Stock-to-part ratio
- Warnings
- Confidence
- Rule identifiers

The user must be able to edit every stock assumption.

Do not assume the smallest bounding box is always machinable.

---

## 5.8 Broad Process Classification

Implement a coarse process classifier before advanced feature recognition.

The initial classifier may suggest:

- Milling candidate
- Turning candidate
- Mill-turn candidate
- Manual-review required
- Mesh-only low-confidence case

Possible evidence includes:

- Rotational symmetry
- Bounding-box proportions
- Cylindrical envelope
- Number of distinct machining directions
- Surface complexity
- Presence of obvious non-rotational regions

The result must include:

- Suggested process
- Alternative processes
- Evidence
- Confidence
- Warnings

The estimator must make the final decision.

---

## 5.9 Setup Suggestion

Implement a rough setup suggestion for the first vertical slice.

It may use:

- Broad process class
- Candidate stock form
- Number of significant machining directions
- Need for part flip
- Obvious secondary features
- Rotational symmetry
- User-selected machine class

The initial system may produce a setup count and high-level orientation suggestions without complete feature recognition.

Every suggestion must be editable.

Store automatic and approved setup plans separately.

Re-analysis must not overwrite a human-approved plan.

---

## 5.10 Feeds-and-Speeds Foundation

Implement a versioned baseline feeds-and-speeds library.

It should support:

- Material family
- Material grade
- Condition or hardness
- Tool family
- Tool diameter range
- Tool material
- Coating
- Operation type
- Roughing versus finishing
- Conservative, standard, and aggressive profiles
- Surface speed
- Chip load
- Feed per revolution
- Axial depth of cut
- Radial width of cut
- Source
- Effective date
- Approval status

For the initial milestone, use conservative baseline data and clearly label it.

The user must be able to override:

- Surface speed
- Spindle speed
- Chip load
- Feed rate
- Depth of cut
- Width of cut
- Material-removal rate

Every override requires a reason if it changes an approved estimate.

---

## 5.11 Coarse Runtime Estimation

Implement a model-informed coarse runtime estimator.

It must not use removed volume alone.

At minimum, consider:

- Removed volume
- Material
- Selected machine class
- Baseline material-removal rate
- Surface-complexity factor
- Tool-access factor
- Small-feature factor when known
- Setup count
- Roughing time
- Finishing allowance
- Tool-change allowance
- Rapid-motion allowance
- Load and unload time
- Probing allowance
- Operator-intervention allowance
- Inspection-at-machine allowance

Produce separate values for:

- Cutting time
- Non-cutting time
- Setup time
- Programming time
- Prove-out time
- Total cycle time
- Total lot time

Every result must expose:

- Formula or rule ID
- Inputs
- Intermediate values
- Output
- Unit
- Confidence
- Warning set
- Runtime-estimation version
- Feeds-and-speeds version

Do not imply CAM-level accuracy.

Clearly label the method, such as:

- Coarse volumetric
- Geometry-adjusted
- Feature-based
- Simplified toolpath
- Imported CAM
- Historical actual

---

## 5.12 Domain Entities

Implement the minimum entities required by the active vertical slice.

Likely entities:

- Customer
- RFQ
- Quote
- QuoteRevision
- Part
- PartRevision
- ModelFile
- ModelAnalysis
- GeometryWarning
- QuantityOption
- MaterialSelection
- StockDefinition
- ProcessSuggestion
- SetupPlan
- Routing
- Operation
- Workcenter
- ToolAssumption
- FeedSpeedAssumption
- RuntimeEstimate
- CostComponent
- PricingPolicy
- Assumption
- Risk
- Override
- RateCard

Favor aggregates and invariants over uncontrolled public mutation.

Examples:

- Deliver quantity must be positive.
- Make quantity cannot be below deliver quantity.
- A quote revision cannot silently overwrite an approved revision.
- A manual override requires a reason.
- A price below an enforced floor requires authorization.
- An operation must identify its estimate method.
- Every calculated amount must identify its source.
- A model with unresolved units cannot produce an approved estimate.
- An approved routing cannot be silently replaced by automatic analysis.
- Mesh-derived analysis must retain its lower-confidence classification.

---

## 5.13 Estimation Engine

Implement the estimation engine independently of the UI.

Use pure functions or explicit calculation services where practical.

The engine must produce a structured result rather than only a final price.

A result should include:

- Material cost
- Programming cost
- Setup cost
- Prove-out cost
- Cutting-machine cost
- Non-cutting-machine cost
- Direct labor
- Tooling
- Fixtures
- Inspection
- Quality documentation
- Outside processing
- Freight
- Packaging
- Non-recurring cost
- Scrap allowance
- Rework allowance
- Risk reserve
- Overhead
- Total internal cost
- Selling price
- Unit price
- Quantity-specific results
- Warnings
- Confidence explanation
- Calculation trace

Each result node should expose:

- Rule ID
- Input values
- Intermediate values
- Output value
- Unit
- Rounding behavior
- Source type
- Rate-card version
- Feeds-and-speeds version
- Geometry-analysis version
- Runtime-estimation version
- Override state

Do not bury formulas inside UI event handlers.

---

## 5.14 Golden Model and Estimate Fixtures

Create reviewed fixtures.

Start with at least:

- Simple rectangular aluminum milled part in STEP
- Similar part in STL
- Cylindrical turned part
- Part with obvious secondary milling
- Part with high surface complexity
- Aerospace part requiring FAI
- Part with an outside process
- Multi-quantity quote
- Model with unit ambiguity
- Invalid or non-watertight mesh
- Model requiring geometry healing

Store expected:

- Import status
- Units
- Bounding dimensions
- Volume
- Surface area
- Stock suggestion
- Process suggestion
- Setup suggestion
- Runtime estimate
- Cost result
- Warning set
- Confidence

Do not update expected results only to make tests pass. Investigate the cause of every change.

---

## 5.15 Persistence

Implement persistence behind repository interfaces or application services.

For standalone mode, use the approved local database, likely SQLite.

Include:

- Migrations
- Transactions
- Stable identifiers
- Created and updated timestamps
- Quote revisions
- Model file metadata
- Model hashes
- Model-analysis snapshots
- Setup-plan snapshots
- Runtime-estimate snapshots
- Rate-card versions
- Feeds-and-speeds versions
- Calculation versions
- Override history
- Attachment metadata
- Backup and restore foundation

Do not store money as floating point.

Do not let UI components execute arbitrary SQL.

Test:

- Save and reload
- Analysis snapshot preservation
- Model revision changes
- Migration from earlier schemas
- Transaction rollback
- Backup creation
- Restore validation

---

## 5.16 Application Services

Create use cases that coordinate domain logic and persistence.

Possible first use cases:

- Create RFQ
- Create customer
- Add part
- Import model
- Confirm model units
- Analyze model
- Approve stock suggestion
- Select process
- Approve setup plan
- Add quantity option
- Select material
- Edit feeds and speeds
- Calculate runtime
- Add routing operation
- Calculate estimate
- Save draft
- Revise quote
- Apply override
- Generate internal summary
- Generate customer quote preview

UI components should call application services rather than coordinating geometry, database, and calculation internals directly.

---

## 5.17 Design-System Foundation

Before building many screens, implement approved design-system primitives.

Include:

- Application surfaces
- Text styles
- Buttons
- Icon buttons
- Inputs
- Selectors
- Search fields
- Tables
- Tree controls
- Tabs
- Segmented controls
- Badges
- Tooltips
- Popovers
- Context menus
- Inspector sections
- Cost-summary components
- Confidence indicators
- Warnings
- Validation messages
- Empty states
- Loading states
- Toasts
- Dialogs
- Split panes
- Resizable panels
- Keyboard focus treatment
- Viewport overlays
- Model-selection state

Create a component gallery or design-system preview screen in development builds.

Use tokens. Avoid scattered literal styles.

Do not make the application look like an unmodified web page embedded in a desktop shell.

---

## 5.18 Application Shell

Build the approved application shell.

Likely elements:

- Custom application frame
- Native-compatible window controls
- Primary navigation
- Workspace header
- Command palette
- Global search
- Recent quotes
- Main content region
- Model viewport
- Contextual inspector
- Persistent estimate summary
- Analysis-status indicator
- Theme support
- Density preference
- Autosave status

Plan keyboard shortcuts for:

- New quote
- Open
- Import model
- Save
- Search
- Command palette
- Fit model to view
- Standard model views
- Add operation
- Duplicate
- Undo
- Redo
- Navigate panels
- Recalculate
- Preview quote

Do not override established operating-system shortcuts without strong reason.

---

## 5.19 Quote and Model Workspace

Build a cohesive quote workspace rather than disconnected forms.

A possible layout:

- RFQ and customer header
- Part and revision navigation
- Main 3D viewport
- Model and feature tree
- Routing panel
- Contextual property inspector
- Persistent cost and price summary
- Assumptions and warning drawer
- Quantity comparison
- Approval status

The estimator should move efficiently among:

- Model
- Geometry warnings
- Stock
- Process
- Setups
- Features
- Tools
- Feeds and speeds
- Routing
- Inspection
- Outside processing
- Costs
- Pricing
- Risks
- Quote preview

Favor inline editing and progressive disclosure over repeated modal dialogs.

---

## 5.20 Material Selection

Implement a minimal material library and selection workflow.

The first version should support:

- Material family
- Grade or alloy
- Specification
- Condition or temper
- Density
- Stock form
- Blank dimensions
- Supplier
- Unit price
- Minimum order
- Cut charge
- Freight
- Certification charge
- Material yield
- Effective date
- Notes

Calculate:

- Required stock
- Gross purchased material
- Part mass
- Stock mass
- Removed mass
- Yield
- Material cost
- Per-part allocation
- Scrap allowance

Show all assumptions.

---

## 5.21 Routing Editor

Implement an operation-based routing editor.

The first version should allow:

- Add
- Delete
- Duplicate
- Reorder
- Choose workcenter
- Choose operation type
- Select source: automatic, template, imported, historical, or manual
- Assign setup
- Enter setup time
- Enter programming time
- Enter cutting time
- Enter non-cutting time
- Enter labor
- Enter tooling
- Enter fixture costs
- Enter inspection
- Enter outside-process cost
- Enter batch size
- Enter notes
- Apply an override

Each operation must display its contribution to total cost.

Reordering operations must preserve stable identities.

---

## 5.22 Quantity Breaks

Calculate multiple requested quantities from the same estimate.

Correctly handle:

- One-time costs
- Recurring costs
- Per-lot costs
- Per-part costs
- Setup amortization
- Fixture amortization
- Material yield
- Make quantity
- Scrap quantity
- Outside-process minimums
- Tool replacement
- Price rounding
- Minimum line-item prices

Present a comparison table showing:

- Requested quantity
- Make quantity
- Total cost
- Unit cost
- Selling price
- Unit price
- Margin
- Lead time when available
- Warnings
- Confidence

---

## 5.23 Margin and Markup

Implement markup and margin as distinct concepts.

The UI must clearly identify which method is active.

When converting:

- Use documented formulas.
- Display the effective corresponding value.
- Prevent invalid division.
- Test boundary cases.
- Record manual price overrides.

---

## 5.24 Explainability UI

Every important total must be inspectable.

The user should be able to answer:

- Why is the price this amount?
- What did the model analysis detect?
- Which stock was assumed?
- Why was this process selected?
- Why were this many setups proposed?
- Which feeds and speeds were used?
- Which runtime method was used?
- Which costs are one-time?
- Which costs recur per part?
- Which values were overridden?
- Which rates were used?
- Which values have low confidence?
- Which requirements are still missing?

Implement drill-down from total price to:

- Cost categories
- Routing operations
- Setups
- Runtime components
- Tools
- Feeds and speeds
- Geometry assumptions
- Individual calculation rules

---

## 5.25 Save, Reopen, and Revision Safety

A saved quote must reopen with the same authoritative result when:

- Inputs have not changed
- The same model-analysis snapshot is used
- The same setup plan is used
- The same feeds-and-speeds version is used
- The same rate-card version is used
- The same calculation version is used

Do not silently reanalyze an approved quote using new algorithms.

Do not silently recalculate an approved quote using edited rates.

When the source model changes:

- Create or identify a new model revision
- Compute a new hash
- Preserve the old model
- Compare metadata where possible
- Require explicit re-analysis
- Preserve the prior approved quote revision

---

## 5.26 Quote Preview

Create separate presentation models for:

- Internal estimate detail
- Customer-facing quote

The customer-facing quote must not expose:

- Internal machine rates
- Labor rates
- Internal margins
- Internal risk notes
- Geometry-analysis internals
- Sensitive assumptions
- Controlled filenames

Support configurable:

- Company information
- Customer information
- Quote number
- Revision
- Validity date
- Part number
- Part revision
- Quantities
- Unit and extended prices
- Lead times
- Non-recurring charges
- Assumptions
- Exclusions
- Terms
- Signature or approval fields

---

## 6. Feature-Recognition Development Sequence

After the first vertical slice, implement feature recognition incrementally.

### Phase A — Primitive recognition

- Planar faces
- Cylindrical faces
- Conical faces
- Through holes
- Blind holes
- Basic pockets
- Basic slots

### Phase B — Process-oriented recognition

- Counterbores
- Countersinks
- Spotfaces
- Steps
- Bosses
- Chamfers
- Fillets
- Turning diameters
- Grooves
- Tapers
- Cutoff features

### Phase C — Accessibility and setup planning

- Machining direction
- Tool access
- Reach
- Undercuts
- Deep cavities
- Thin walls
- Multi-axis need
- Setup grouping

### Phase D — Advanced process recognition

- Live-tool features
- Mill-turn suitability
- Swiss suitability
- Five-axis accessibility
- EDM indicators
- Grinding indicators
- Custom-tool indicators

Every phase requires fixtures, expected results, confidence behavior, and documented limitations.

---

## 7. Runtime-Estimation Development Sequence

### Level 1 — Coarse volumetric

- Removed volume
- Baseline material-removal rate
- Basic setup and overhead allowances

### Level 2 — Geometry adjusted

- Surface complexity
- Part proportions
- Tool-access difficulty
- Finishing allowance
- Small-feature penalty

### Level 3 — Feature based

- Operation-specific tools
- Tool engagement
- Feature dimensions
- Drilling and turning cycles
- Tool changes
- Setup grouping

### Level 4 — Simplified toolpath

- Approximate path lengths
- Stepdown
- Stepover
- Entry and exit
- Rapids
- Retracts
- Rest machining

### Level 5 — Imported CAM or NC data

- Imported cycle time
- Machine acceleration adjustments
- Tool changes
- Overrides
- Source provenance

### Level 6 — Historical calibration

- Machine-specific correction
- Material-specific correction
- Tool-specific correction
- Setup-specific correction
- Confidence from similar jobs

Do not skip directly to a complex toolpath engine before validating simpler levels.

---

## 8. Security Implementation Rules

Treat models, drawings, and specifications as potentially sensitive.

Do not:

- Upload them to external services
- Include filenames or geometry in telemetry
- Log document contents
- Create unrestricted temporary copies
- Allow arbitrary UI-origin filesystem access
- Display a compliance badge based only on settings
- Send model data to external AI systems by default

Implement least privilege for:

- Opening files
- Saving files
- Exporting quotes
- Accessing attachment directories
- Backup and restore
- Geometry worker processes

Add audit events as features enter scope:

- Model import
- Unit confirmation
- Geometry healing
- Model revision
- Quote approval
- Quote revision
- Rate-card change
- Feeds-and-speeds change
- Price override
- Runtime override
- Cost override
- Document export
- Permission change
- Classification change
- Backup and restore

---

## 9. Cross-Platform Requirements

Do not postpone cross-platform verification.

Maintain CI or build checks for:

- Windows
- Linux
- macOS

Document platform-specific dependencies.

Test:

- Window sizing
- High-DPI scaling
- Font rendering
- GPU rendering
- Model-viewer behavior
- File dialogs
- Keyboard shortcuts
- Custom title bar
- Menus
- Drag and drop
- Clipboard
- PDF export
- Theme detection
- Path handling
- Application data directories
- Installer generation
- Native geometry dependencies

Isolate platform-specific code in the platform module.

Never assume Windows path separators or case-insensitive filesystems.

---

## 10. Performance Expectations

The application should feel immediate.

Target:

- Fast startup
- Immediate local navigation
- Responsive model manipulation
- Non-blocking import and analysis
- Cancellation for long analysis
- Debounced but rapid estimate recalculation
- Smooth tables
- Efficient autosave
- Clear progress for model import and reporting

Run expensive geometry work off the UI thread.

Do not optimize blindly. Add measurements for:

- Import time
- Geometry-analysis time
- Mesh-generation time
- Runtime-estimation time
- Viewer frame time
- Save and reopen time

Cache derived geometry carefully using model hash and algorithm version.

---

## 11. Error Handling

Use typed errors at module boundaries.

User-facing errors should:

- Explain what failed
- Preserve entered data
- Suggest recovery
- Avoid exposing sensitive internals
- Provide a diagnostic identifier when useful

Geometry errors should distinguish:

- Unsupported format
- Parse failure
- Unit ambiguity
- Invalid solid
- Non-watertight mesh
- Healing failure
- Unsupported entity
- Excessive complexity
- Resource exhaustion
- Cancelled analysis
- Worker failure

Do not use `unwrap()` or `expect()` in production paths unless failure is genuinely impossible and documented.

Do not swallow errors to make the interface appear successful.

---

## 12. Dependency Policy

Before adding a dependency:

- Check whether an existing dependency already covers the need.
- Verify active maintenance.
- Review licensing.
- Review platform support.
- Review security implications.
- Review native build requirements.
- Review redistribution rights.
- Review export restrictions when relevant.
- Document why it is needed.
- Avoid duplicate crates.
- Avoid large frameworks for one small utility.

Geometry and CAD libraries require special scrutiny because they may introduce:

- Native code
- Large binaries
- Platform-specific build systems
- Licensing obligations
- Patent concerns
- Commercial-format restrictions
- Crash risk
- Thread-safety constraints

---

## 13. Definition of Done for a Task

A task is complete only when:

- Acceptance criteria pass.
- Relevant unit tests pass.
- Relevant geometry fixtures pass.
- Relevant golden estimates pass.
- Formatting passes.
- Applicable lint checks pass.
- The application builds.
- The application launches when relevant.
- Documentation is updated.
- Requirement links are updated.
- No hidden placeholder behavior remains.
- No unresolved TODO blocks the stated purpose.
- `docs/PROJECT_STATE.md` is updated.
- The progress log is updated.

---

## 14. Definition of Done for the First CAD-Assisted Vertical Slice

The first vertical slice is complete only when:

- It builds on all three target operating systems through CI or documented evidence.
- A user can create an RFQ and part.
- A user can import STEP, STL, or 3MF.
- Units can be confirmed.
- The model can be viewed.
- Bounding dimensions, volume, and surface area are calculated.
- Material mass is calculated.
- A stock suggestion is produced.
- Removed volume is calculated.
- A broad process suggestion is produced.
- A rough setup suggestion is produced.
- A coarse runtime estimate is produced.
- Runtime is separated into cutting and non-cutting components.
- The estimate can be manually edited.
- Multiple quantities are calculated.
- Material, programming, setup, run, labor, inspection, tooling, and outside-process costs are represented.
- Cost and price are separated.
- Margin and markup are distinguished.
- Totals are explainable.
- Overrides require reasons.
- Model hash and analysis versions are preserved.
- Rate and calculation versions are preserved.
- Golden geometry and calculation tests pass.
- The estimate can be saved and reopened.
- An internal summary can be viewed.
- A customer-facing quote can be previewed.
- The UI follows the approved design system.
- Major keyboard workflows function.
- Documentation accurately reflects implementation.

---

## 15. Session Completion Report

Before ending each development session:

1. Run relevant build and test commands.
2. Update `docs/PROJECT_STATE.md`.
3. Add an entry to `docs/07-delivery/progress-log.md`.
4. Update completed tasks and requirements.
5. Record new risks or technical debt.
6. Identify the next exact task.

Report:

- Work completed
- Files changed
- Requirements addressed
- Model fixtures added or changed
- Tests added
- Test and build results
- Geometry capabilities added
- Runtime capabilities added
- Screens or workflows added
- Architectural decisions made
- Deviations from the plan
- Remaining issues
- Exact next action

Do not claim success when a target platform, model format, or acceptance criterion has not been verified.

Begin now by reading the project guidance and implementing the next approved task. When the repository contains only planning documents and no production code, begin with the foundation milestone and build the smallest real end-to-end path toward the approved CAD-assisted vertical slice.
