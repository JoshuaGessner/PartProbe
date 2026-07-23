# Prompt 1 — Project Setup, Research, and Deep Planning

You are the principal software architect, senior Rust engineer, manufacturing-estimation specialist, CAD/CAM systems designer, product strategist, UX systems designer, and security-conscious technical lead for a new specialty machine-shop estimating application.

Your first responsibility is not to rush into production implementation. Your responsibility is to establish the repository, documentation system, domain model, technical architecture, research record, validation strategy, and phased path forward that every later agent must follow.

The finished product will be a CAD-assisted estimating application for specialty machining work. It must analyze imported 3D models, propose a plausible manufacturing approach, estimate machining and supporting costs, and give a human estimator a transparent, editable starting point.

The system is an estimating assistant, not an unquestionable automatic quote generator and not a replacement for production CAM.

---

## 1. Product Mission

Design a robust, efficient, modern desktop application for estimating and quoting work in a specialty machine shop.

The shop performs a broad mix of work, including:

- Aerospace components
- Defense-related contracts
- Commercial and industrial components
- Prototype work
- Repair and one-off work
- Repeat production
- Short-run and medium-run production
- Conventional milling and turning
- Multi-axis milling
- Live-tool turning
- Mill-turn work
- Swiss work
- Manual machining
- Grinding and EDM when applicable
- Work involving difficult, expensive, scarce, or tightly controlled materials
- Work requiring substantial inspection, documentation, traceability, or outside processing

The application must:

- Run on Windows, Linux, and macOS
- Be written primarily in Rust
- Use a modern, custom-designed desktop interface
- Avoid the appearance of a generic Windows business application
- Remain efficient for expert estimators while still being approachable
- Preserve explainability and auditability
- Work well in local-first and restricted-data environments
- Analyze 3D model files as a core part of the estimating workflow
- Support manual correction of every automatically generated assumption
- Preserve the distinction between estimated cost, risk allowance, and selling price

The application should take broad interaction and design inspiration from high-end CAD/CAM tools, Figma, Linear, professional analytics products, and premium engineering software without copying their branding, layout, or proprietary visual identity.

The desired experience is:

- Modern
- Sleek
- Dense without feeling cramped
- Keyboard-friendly
- Fast
- Highly legible
- Ergonomic for long work sessions
- Consistent across operating systems
- Visually custom
- Professional enough for aerospace and defense quoting
- Efficient enough for a busy estimator
- Explainable rather than mysterious

---

## 2. Defining Product Capability

The defining workflow is:

1. Import a 3D model.
2. Validate units, scale, file integrity, and geometry.
3. Extract basic geometric properties.
4. Suggest a stock form and starting blank.
5. Analyze likely machining features and tool access.
6. Suggest manufacturing processes, machines, orientations, and setups.
7. Build a proposed operation routing.
8. Estimate cutting and non-cutting time.
9. Estimate material, setup, programming, tooling, inspection, outside processing, documentation, and other costs.
10. Present a confidence-rated and fully editable estimate.
11. Allow the estimator to change any assumption.
12. Preserve the original automatic result and every manual override.
13. Compare future actual results against the estimate without silently rewriting approved rules.

The software must be useful when only a 3D model is available, but it must clearly identify the important information that a model usually cannot provide, such as tolerances, GD&T, surface finish, threads, material specifications, inspection requirements, heat treatment, coatings, certifications, customer clauses, and export-control requirements.

---

## 3. Primary Product Principles

### 3.1 No unexplained black box

Every estimate must be reproducible and traceable to:

- Imported files and file revisions
- Model units and scale
- Geometry measurements
- Detected features
- Stock assumptions
- Material assumptions
- Manufacturing-process assumptions
- Setup and orientation assumptions
- Tool assumptions
- Feeds and speeds
- Machine capabilities
- Machine and labor rates
- Programming assumptions
- Inspection assumptions
- Outside-process assumptions
- Scrap and rework assumptions
- Risk allowances
- Pricing rules
- Manual overrides
- The version of the geometry-analysis algorithm
- The version of the feature-recognition algorithm
- The version of the runtime-estimation algorithm
- The version of the shop rate card
- The version of the feeds-and-speeds library

Machine learning or statistical suggestions may eventually assist the estimator, but they must never silently replace deterministic calculations, approved shop settings, or human judgment.

### 3.2 Human-in-the-loop estimation

Automatic analysis produces a proposed plan, not final truth.

The user must be able to:

- Accept, reject, or edit detected features
- Change stock form and dimensions
- Change process selection
- Change machine assignment
- Change setup count
- Change part orientations
- Change tool choices
- Change feeds and speeds
- Add omitted operations
- Remove irrelevant operations
- Add drawing-driven requirements
- Record uncertainty
- Override time or cost values with a reason

### 3.3 Estimating rather than production CAM

The system may estimate simplified tool motion and process effort, but it must not claim to generate safe production-ready G-code unless that becomes a separately planned and validated future product.

### 3.4 Local-first and controlled data handling

CAD files, drawings, and specifications may contain sensitive technical data. The default design must not upload them to external services, analytics platforms, telemetry providers, or external AI systems.

---

## 4. Immediate Assignment

Inspect the repository before making changes.

If the repository is empty, initialize an appropriate project structure and documentation tree. Do not begin broad production feature development during this planning phase.

You may create:

- Repository configuration
- Documentation files
- Architecture prototypes
- Geometry-processing technical spikes
- UI technical spikes
- Non-production model-viewer experiments
- Calculation experiments
- Feature-recognition experiments
- Test fixtures
- Research notes
- Benchmarks
- Mockups
- Sample estimate data

Do not build major production modules until the planning-readiness criteria in this prompt have been satisfied.

---

## 5. Documentation Governance

Create the following root files:

- `README.md`
- `AGENTS.md`
- `CHANGELOG.md`
- `docs/INDEX.md`
- `docs/PROJECT_STATE.md`

Organize the remaining documentation approximately as follows. Adjust the structure only when a documented reason exists.

```text
docs/
├── INDEX.md
├── PROJECT_STATE.md
├── 00-product/
│   ├── vision.md
│   ├── users-and-personas.md
│   ├── scope.md
│   ├── non-goals.md
│   ├── success-metrics.md
│   └── terminology.md
├── 01-research/
│   ├── research-plan.md
│   ├── machining-estimation.md
│   ├── cad-file-formats.md
│   ├── geometry-kernels.md
│   ├── feature-recognition.md
│   ├── runtime-estimation.md
│   ├── feeds-and-speeds.md
│   ├── setup-and-orientation-planning.md
│   ├── aerospace-quality.md
│   ├── defense-data-handling.md
│   ├── competitor-analysis.md
│   ├── rust-ui-evaluation.md
│   └── sources.md
├── 02-domain/
│   ├── domain-overview.md
│   ├── estimating-model.md
│   ├── calculation-rules.md
│   ├── geometry-analysis-model.md
│   ├── feature-model.md
│   ├── stock-selection-model.md
│   ├── material-model.md
│   ├── tooling-model.md
│   ├── feeds-and-speeds-model.md
│   ├── machine-and-workcenter-model.md
│   ├── routing-and-operation-model.md
│   ├── quality-and-inspection-model.md
│   ├── pricing-model.md
│   ├── quote-lifecycle.md
│   ├── actuals-and-calibration.md
│   └── glossary.md
├── 03-requirements/
│   ├── functional-requirements.md
│   ├── nonfunctional-requirements.md
│   ├── requirements-matrix.md
│   ├── workflows.md
│   ├── permissions-matrix.md
│   ├── cad-import-requirements.md
│   ├── model-analysis-requirements.md
│   ├── import-export-requirements.md
│   └── reporting-requirements.md
├── 04-architecture/
│   ├── system-overview.md
│   ├── module-boundaries.md
│   ├── data-model.md
│   ├── estimation-engine.md
│   ├── geometry-engine.md
│   ├── feature-recognition-pipeline.md
│   ├── model-viewer.md
│   ├── persistence.md
│   ├── desktop-platform.md
│   ├── security-model.md
│   ├── deployment-models.md
│   ├── integration-strategy.md
│   ├── observability.md
│   └── adr/
├── 05-ux/
│   ├── experience-principles.md
│   ├── information-architecture.md
│   ├── design-system.md
│   ├── interaction-patterns.md
│   ├── model-review-workflow.md
│   ├── feature-review-workflow.md
│   ├── accessibility.md
│   ├── screen-inventory.md
│   ├── workflow-wireframes.md
│   └── usability-test-plan.md
├── 06-quality/
│   ├── test-strategy.md
│   ├── calculation-validation.md
│   ├── geometry-validation.md
│   ├── model-fixture-strategy.md
│   ├── feature-recognition-validation.md
│   ├── runtime-estimation-validation.md
│   ├── test-data-strategy.md
│   ├── security-testing.md
│   ├── cross-platform-testing.md
│   └── release-acceptance.md
├── 07-delivery/
│   ├── roadmap.md
│   ├── milestones.md
│   ├── backlog.md
│   ├── risk-register.md
│   ├── dependencies.md
│   ├── release-plan.md
│   └── progress-log.md
└── 08-decisions/
    ├── assumptions.md
    ├── open-questions.md
    ├── deferred-items.md
    └── decision-log.md
```

---

## 6. Documentation Metadata

Place a metadata section near the beginning of every important document containing:

- Status: Draft, In Review, Approved, Superseded, or Deferred
- Last updated date
- Related requirement IDs
- Related architecture decision IDs
- Open questions
- Dependencies
- Supersedes or superseded-by links when applicable

Do not duplicate authoritative information throughout the repository. Choose one canonical document and link to it from other documents.

---

## 7. Documentation Index Requirements

`docs/INDEX.md` must provide:

1. A concise product description.
2. Links to every major document.
3. The current development phase.
4. The current milestone.
5. Overall project status.
6. Documents awaiting review.
7. Open blocking decisions.
8. Recently completed work.
9. The next recommended actions.
10. A requirements coverage summary.
11. A geometry-analysis coverage summary.
12. A roadmap summary.
13. A risk summary.
14. A changelog link.
15. Instructions for future agents.
16. A definition of which documents are authoritative for each subject.
17. A list of required technical spikes and their status.
18. The current first-class and experimental CAD file formats.

Use stable identifiers:

- `REQ-F-###` for functional requirements
- `REQ-NF-###` for nonfunctional requirements
- `UX-###` for UX requirements
- `SEC-###` for security requirements
- `CALC-###` for calculation rules
- `GEO-###` for geometry-analysis rules
- `FEAT-###` for feature-recognition rules
- `TIME-###` for runtime-estimation rules
- `DATA-###` for data requirements
- `ADR-####` for architecture decisions
- `RISK-###` for risks
- `EPIC-###` for epics
- `TASK-###` for implementation tasks
- `TEST-###` for validation requirements

Every implementation task must link to one or more requirements. Every completed requirement must link to tests or other validation evidence.

---

## 8. Project-State File

`docs/PROJECT_STATE.md` must be optimized for rapid handoff between coding agents.

It must contain:

- Current phase
- Current milestone
- Current branch, when known
- Last completed task
- Work currently in progress
- Next task
- Known build status
- Known test status
- Active technical debt
- Active blockers
- Open architectural decisions
- Current supported model formats
- Current geometry-analysis capabilities
- Current runtime-estimation capabilities
- Current model fixtures used for validation
- Files most likely to be changed next
- Commands needed to build and test
- A concise “read these first” list

Keep this document brief and current. It is not a historical log.

Use `docs/07-delivery/progress-log.md` for historical session records.

---

## 9. AGENTS.md Requirements

Create a root-level `AGENTS.md` containing mandatory instructions for every future agent.

Include these rules:

1. Read `docs/INDEX.md` and `docs/PROJECT_STATE.md` before changing code.
2. Read relevant requirements, architecture, UX, geometry, and calculation documents before modifying a module.
3. Do not contradict an approved document without creating or updating an ADR.
4. Do not silently change calculation behavior.
5. Do not silently change geometry interpretation.
6. Every calculation change requires:
   - A documented rule
   - Tests
   - A versioning or migration decision
   - Updated worked examples
7. Every geometry-analysis change requires:
   - Documented behavior
   - Model fixtures
   - Expected measurements
   - Regression tests
   - Confidence behavior
8. Every feature-recognition change requires:
   - Test parts
   - Expected detections
   - Known false positives and false negatives
   - Confidence criteria
9. Every data-schema change requires:
   - A migration
   - Backward-compatibility consideration
   - Updated documentation
10. Every UI feature must use the project design system.
11. Do not introduce default-looking widgets without deliberate styling.
12. Do not add dependencies without documenting the reason, maintenance condition, licensing, and security implications.
13. Avoid platform-specific behavior unless isolated behind an interface.
14. Keep Windows, Linux, and macOS support intact.
15. Keep the application runnable at the end of each implementation session.
16. Update `PROJECT_STATE.md`, the progress log, and affected requirement statuses after meaningful work.
17. Do not claim a feature is complete unless acceptance criteria pass.
18. Do not describe the product as AS9100, CMMC, ITAR, NIST, or export-control compliant without a formal external determination.
19. Do not send CAD files, drawings, part models, specifications, prices, or estimate data to external services by default.
20. Do not add AI functionality that transmits controlled files externally without an explicit ADR and deployment policy.
21. Prefer deterministic and explainable calculations.
22. Manual overrides must record the original value, new value, user, time, and reason.
23. Preserve an audit trail for approved quotes, model revisions, and rate changes.
24. Do not treat mesh-derived feature recognition as equivalent to solid-model recognition.
25. Do not imply that rough runtime estimates are production CAM simulations.
26. Preserve the source model and a hash of the analyzed file.
27. Record model units, scale decisions, healing actions, and import warnings.
28. Never automatically overwrite a human-approved routing after re-analysis.

---

## 10. Research Phase

Conduct focused research before finalizing requirements.

Research at least:

### Machining estimation

- Traditional manual machine-shop estimating
- Operation-based estimating
- Feature-based estimating
- CNC cycle-time estimation
- Milling estimation
- Turning estimation
- Mill-turn estimation
- Swiss estimation
- Sawing and material preparation
- Grinding
- EDM
- Manual machining
- Programming and CAM labor
- Setup and prove-out labor
- Fixturing and soft-jaw costs
- Cutting tools and consumables
- Tool-life assumptions
- Material pricing and yield
- Inspection and documentation costs
- Outside processing
- Lead-time estimation
- Quote approval workflows
- Historical estimate-versus-actual analysis

### CAD and geometry

- STEP import
- IGES import
- STL import
- 3MF import
- OBJ import
- Parasolid support and licensing implications
- ACIS/SAT support and licensing implications
- Native CAD format translation
- B-rep versus mesh limitations
- Geometry healing
- Unit detection
- Solid validation
- Watertightness
- Bounding boxes
- Oriented bounding boxes
- Volume and surface-area computation
- Center of mass
- Stock-envelope generation
- Rotational symmetry detection
- Feature recognition
- Tool-access analysis
- Setup-orientation planning
- Simplified toolpath estimation
- Geometry-kernel options available to Rust
- FFI safety and packaging implications
- Cross-platform native dependency packaging
- Commercial translator options
- File-format licensing restrictions

### Feeds, speeds, and runtime

- Surface-speed calculations
- Chip-load calculations
- Feed-per-revolution calculations
- Tool engagement
- Material-removal rate
- Depth and width of cut
- Machine power and torque constraints
- Tool deflection
- Roughing and finishing strategies
- Drilling, pecking, reaming, tapping, and boring
- Turning cycles
- Tool-change time
- Rapid motion
- Spindle acceleration
- Probing
- Indexing
- Part transfer
- Operator intervention
- Unattended machining
- Tool-life replacement allowances
- Conservative versus aggressive machining profiles

### Aerospace, defense, and quality

- First-article inspection
- Material and process certifications
- Inspection planning
- CMM programming and runtime
- Customer-specific quality clauses
- Traceability
- Source inspection
- Controlled technical data
- CUI handling
- Export-controlled data
- Restricted storage
- Auditability
- Role-based access

### Product and architecture

- Modern digital quoting systems
- Rust cross-platform desktop frameworks
- Rust-compatible 3D rendering
- Rust-compatible geometry kernels
- Cross-platform packaging and signing
- Local-first architecture
- Team/LAN architecture
- Secure document storage
- High-end engineering desktop UX

Record sources and distinguish:

- Formal requirements
- Industry practices
- Vendor claims
- Shop-specific assumptions
- Unverified ideas

Never present vendor marketing as independent fact.

---

## 11. CAD File-Format Strategy

Create an explicit format-support matrix.

At minimum, evaluate:

### First-class initial formats

- STEP
- STL
- 3MF

### Secondary formats

- IGES
- OBJ

### Optional or future formats

- Parasolid
- ACIS/SAT
- SolidWorks
- Inventor
- Creo
- CATIA
- NX
- Fusion exports
- Other native formats

For each format, document:

- Geometry type
- Unit handling
- Metadata availability
- Assembly support
- Color and layer support
- Exact-solid capability
- Mesh limitations
- Import-library options
- Licensing
- Cross-platform packaging difficulty
- Expected analysis quality
- Known failure modes
- Whether the format is accepted directly or requires conversion

STEP should be treated as the likely primary engineering interchange format, but the final decision must be recorded in an ADR after technical evaluation.

Native CAD formats must not be promised without confirming translator availability, licensing, deployment restrictions, and cost.

---

## 12. Geometry-Processing Pipeline

Design a geometry engine with explicit stages.

A candidate pipeline is:

1. File intake
2. File hashing
3. Format identification
4. Unit extraction or user confirmation
5. Parsing
6. Geometry healing where supported
7. Solid and shell validation
8. Mesh validation when applicable
9. Basic-property extraction
10. Candidate orientation generation
11. Stock-envelope analysis
12. Feature extraction
13. Tool-access analysis
14. Process classification
15. Setup proposal
16. Runtime-estimation input generation
17. Confidence scoring
18. User review
19. Approved analysis snapshot

Each stage must produce:

- Structured results
- Warnings
- Confidence
- Timing metrics
- Algorithm version
- Recoverable errors
- Diagnostic information that does not expose sensitive model contents

The architecture must allow the geometry engine to run independently of the UI.

---

## 13. Basic Geometry Requirements

The initial geometry engine should plan for:

- Model units
- Scale
- Axis-aligned bounding box
- Oriented bounding box
- Volume
- Surface area
- Center of mass
- Number of bodies
- Number of shells
- Solid validity
- Watertightness for meshes
- Degenerate geometry
- Non-manifold edges
- Candidate stock dimensions
- Candidate stock orientation
- Minimum enclosing cylindrical stock when appropriate
- Material mass from density
- Removed material volume
- Buy-to-fly or stock-to-part ratio
- Thin-section indicators
- Minimum local thickness where feasible
- Surface-complexity indicators

Every measurement must state its unit and derivation.

---

## 14. Feature Recognition

Plan feature recognition in phases.

### 14.1 Prismatic milling features

- Planar faces
- Pockets
- Open pockets
- Closed pockets
- Slots
- Steps
- Through holes
- Blind holes
- Counterbores
- Countersinks
- Spotfaces
- Bosses
- Chamfers
- Fillets
- Thin walls
- Deep cavities
- Undercuts
- Small internal radii
- Difficult tool-access regions
- Multiple machining directions

### 14.2 Turning features

- Rotational symmetry
- Outside diameters
- Inside diameters
- Faces
- Shoulders
- Grooves
- Bores
- Tapers
- Cutoff features
- Thread candidates
- Cross holes
- Milled flats
- Live-tool features
- Subspindle-transfer candidates

### 14.3 Advanced process indicators

- Five-axis accessibility
- Mill-turn suitability
- Swiss suitability
- EDM-only or EDM-preferred features
- Grinding requirements
- Deep-hole requirements
- Features inaccessible by ordinary three-axis machining
- Features likely requiring custom tooling
- Multi-body or assembly conditions

Every detected feature must have:

- Stable identifier
- Geometry references
- Feature type
- Dimensions
- Candidate tools
- Candidate processes
- Candidate setup orientations
- Confidence
- Warnings
- User acceptance state
- Manual correction history

The UI must allow the user to inspect, accept, reject, merge, split, or redefine detections.

---

## 15. Stock Selection

Plan a stock-selection engine that can suggest:

- Rectangular bar
- Plate
- Round bar
- Tube
- Pipe
- Hex stock
- Sheet
- Forging
- Casting
- Customer-supplied material
- Near-net blank
- Custom preform

The stock engine should consider:

- Model envelope
- Candidate machining orientations
- Saw allowance
- Facing allowance
- Workholding allowance
- Clamp allowance
- Bar remnant
- Plate nesting
- Standard supplier sizes
- Minimum order
- Cut charges
- Material availability
- Certification requirements
- Grain direction
- Country-of-origin restrictions
- Expected scrap
- Material risk
- Reusable remnants
- Outside blanking processes

All suggestions must be editable.

---

## 16. Setup and Orientation Planning

Design a setup-planning system that proposes one or more manufacturing approaches.

Examples:

- Three-axis milling from rectangular stock
- Four-axis indexing
- Five-axis positional machining
- Five-axis simultaneous machining
- Two-operation turning
- Live-tool turning
- Mill-turn
- Swiss
- Saw plus manual machining
- Waterjet or laser blank followed by machining
- Near-net blank followed by finish machining

The setup planner should consider:

- Reachable faces
- Candidate datums
- Workholding surfaces
- Clamp interference
- Tool access
- Undercuts
- Part rigidity
- Thin walls
- Need for soft jaws
- Need for custom fixtures
- Flip operations
- Secondary machining
- Part transfer
- Subspindle use
- Bar feeding
- Tombstones or multi-part fixtures
- Pallet systems
- Automation
- Inspection between setups
- Datum-transfer risk

The result must be presented as a proposed routing with confidence and alternatives, not as a definitive process plan.

---

## 17. Runtime-Estimation Architecture

The runtime model must distinguish cutting time from non-cutting time.

### 17.1 Cutting-time factors

- Tool diameter
- Tool type
- Tool material
- Tool coating
- Number of flutes or inserts
- Surface speed
- Chip load
- Feed per tooth
- Feed per revolution
- Spindle speed
- Machine RPM limit
- Machine feed-rate limit
- Axial depth of cut
- Radial width of cut
- Material-removal rate
- Tool engagement
- Roughing versus finishing
- Entry and exit
- Number of passes
- Rest machining
- Finish allowance
- Drilling cycles
- Pecking
- Reaming
- Tapping
- Boring
- Turning cycles
- Grooving
- Threading
- Cutoff
- Tool deflection
- Machine power
- Machine torque
- Coolant strategy
- Tool overhang
- Tool-life profile

### 17.2 Non-cutting-time factors

- Rapid travel
- Acceleration and deceleration
- Tool changes
- Spindle acceleration
- Indexing
- Rotary movement
- Probing
- Tool measurement
- Part loading
- Part unloading
- Chip clearing
- Part flipping
- Reclamping
- Bar feeding
- Subspindle transfer
- In-process inspection
- Operator checks
- Tool replacement
- Warmup or stabilization where applicable
- Deburring performed at the machine

### 17.3 Estimation levels

Support multiple calculation levels:

1. Coarse volumetric estimate
2. Geometry-complexity adjusted estimate
3. Feature-based estimate
4. Simplified toolpath estimate
5. Imported CAM or NC estimate
6. Historical actual-based estimate

Each operation must show which estimation method produced its value.

### 17.4 Limitations

Do not rely on removed volume alone. Two parts with equal removed volume may have radically different runtimes due to feature size, tool access, finish requirements, tool changes, and setup complexity.

---

## 18. Feeds-and-Speeds Library

Plan a configurable feeds-and-speeds library containing:

- Workpiece material
- Alloy or grade
- Condition
- Hardness
- Tool material
- Tool coating
- Tool family
- Tool diameter
- Tool geometry
- Number of flutes or inserts
- Machine rigidity class
- Spindle capability
- Coolant method
- Roughing profile
- Finishing profile
- Conservative profile
- Standard profile
- Aggressive profile
- Surface speed
- Chip load
- Feed per revolution
- Axial depth of cut
- Radial width of cut
- Maximum engagement
- Tool-life assumption
- Source
- Effective date
- Approval status

The software may ship with conservative baseline values, but they must be clearly identified as baseline data rather than universal truth.

The shop must be able to calibrate values by:

- Material
- Machine
- Tool family
- Operation type
- Programmer preference
- Historical result

Changes to approved feeds and speeds must be versioned.

---

## 19. Model and Drawing Relationship

Document that 3D models usually cannot fully communicate:

- Tolerances
- GD&T
- Surface finish
- Threads
- Material specification
- Heat treatment
- Coatings
- Plating
- Passivation
- Inspection requirements
- First-article requirements
- Serialization
- Traceability
- Special processes
- Packaging
- Customer quality clauses
- Source inspection
- Export-control classification

The product should support an input package containing:

- 3D model
- Drawing
- Specifications
- RFQ information
- Quantity
- Material
- Customer requirements

Plan a requirement-review workflow that lets users add missing drawing-driven requirements to the model-derived estimate.

Automatic PDF or drawing interpretation may be considered later, but it must not silently create authoritative requirements.

---

## 20. Users and Workflow Discovery

Define at least these user roles:

- Estimator
- Senior estimator
- Machinist
- CNC programmer
- Manufacturing engineer
- Quality inspector
- Quality manager
- Purchasing
- Sales
- Shop manager
- Administrator
- Read-only reviewer

Document workflows for:

- Receiving an RFQ
- Creating a customer and contact
- Importing a 3D model
- Importing a drawing
- Confirming model units
- Resolving geometry warnings
- Reviewing stock suggestions
- Reviewing detected features
- Selecting a manufacturing approach
- Reviewing setup proposals
- Assigning machines
- Editing feeds and speeds
- Adding manual operations
- Recording drawing requirements
- Selecting material
- Estimating tooling and fixturing
- Estimating inspection
- Collecting outside-process pricing
- Creating quantity breaks
- Applying risk and contingency
- Reviewing confidence
- Applying pricing rules
- Obtaining internal approval
- Generating a customer quote
- Revising a quote
- Re-analyzing a revised model
- Comparing model revisions
- Converting a won quote into production-planning data
- Recording actual results
- Comparing actual versus estimated results
- Updating future assumptions through controlled approval

---

## 21. Core Domain Scope

The planning documents must address the following entities and concepts.

### 21.1 Customers, RFQs, and quotes

- Customer
- Customer contact
- Customer-specific pricing policy
- RFQ
- RFQ due date
- Requested delivery
- Quote
- Quote revision
- Quote status
- Bid/no-bid decision
- Decline reason
- Validity period
- Terms
- Exclusions
- Attachments
- Internal notes
- Customer-facing notes

### 21.2 Parts, models, and revisions

- Part number
- Revision
- Description
- Make or buy
- Detail part
- Assembly
- Parent-child relationships
- Model file
- Model format
- Model hash
- Drawing file
- Specification files
- Imported units
- Confirmed units
- Model healing actions
- Geometry warnings
- Model-analysis snapshot
- Feature-analysis snapshot
- Quantity options
- Deliver quantity
- Make quantity
- Spare or destructive-test quantity
- Historical revisions
- Similar-part references

### 21.3 Materials

Support a configurable materials library containing:

- Material family
- Alloy
- Grade
- Specification
- Temper
- Condition
- Hardness
- Stock form
- Available sizes
- Density
- Machinability characteristics
- Supplier
- Supplier part number
- Price
- Price effective date
- Minimum order
- Cut charge
- Freight
- Certification charge
- Lead time
- Lot restrictions
- Country-of-origin restrictions
- Stock on hand
- Remnant availability
- Blank dimensions
- Saw allowance
- Facing allowance
- Kerf
- Nesting or yield
- Scrap value
- Material risk
- Material price history

Do not hard-code a narrow list of common materials.

### 21.4 Tools and holders

Support:

- Tool family
- Tool type
- Diameter
- Reach
- Flute length
- Number of flutes
- Insert geometry
- Tool material
- Coating
- Holder
- Stickout
- Cost
- Expected life
- Tool-change allowance
- Compatible materials
- Compatible machines
- Preferred operation types

### 21.5 Machines and workcenters

Support configurable workcenters such as:

- Three-axis mills
- Four-axis mills
- Five-axis mills
- CNC lathes
- Live-tool lathes
- Mill-turn machines
- Swiss lathes
- Manual mills
- Manual lathes
- Saws
- Grinders
- Wire EDM
- Sinker EDM
- Routers
- Inspection equipment
- CMMs
- Deburring stations
- Cleaning stations
- Packaging stations

Machine data should represent:

- Work envelope
- Axis configuration
- Spindle speed
- Power
- Torque curve where available
- Feed limits
- Rapid rates
- Acceleration assumptions
- Tool capacity
- Bar capacity
- Rotary capability
- Accuracy capability
- Typical utilization
- Setup labor rate
- Run labor rate
- Machine rate
- Burdened rate
- Unattended-running policy
- Minimum charge
- Work calendar
- Qualification restrictions
- Customer restrictions
- Material restrictions
- Process capability
- Default setup allowances
- Default tool-change time
- Default load and unload time
- Default probing time
- Cost-center relationship

Separate physical capability from financial rates.

### 21.6 Routings and operations

Represent a quote as a routing composed of ordered operations.

Possible operations include:

- Material procurement
- Sawing
- Blanking
- Waterjet or laser
- Milling
- Turning
- Mill-turn
- Swiss machining
- Manual machining
- Grinding
- Wire EDM
- Sinker EDM
- Broaching
- Honing
- Lapping
- Deburring
- Cleaning
- In-process inspection
- Final inspection
- CMM inspection
- First-article inspection
- Heat treatment
- Plating
- Anodizing
- Passivation
- Painting
- Nondestructive testing
- Welding
- Assembly
- Marking
- Serialization
- Packaging
- Shipping
- Source inspection
- Administrative documentation

Each operation must support:

- Source: automatic, template, imported, historical, or manual
- Workcenter
- Setup
- Setup orientation
- Assigned features
- Tool set
- Setup time
- Programming time
- Prove-out time
- First-piece time
- Cutting time
- Non-cutting time
- Cycle time
- Load and unload time
- Batch size
- Queue or lead time
- Direct labor
- Machine time
- Unattended time
- Inspection frequency
- Scrap probability
- Rework allowance
- Tooling
- Consumables
- Fixtures
- Outside vendor
- Minimum lot charges
- Freight
- Notes
- Assumptions
- Confidence
- Manual overrides

---

## 22. Complexity and Risk

Do not use one universal complexity multiplier.

Model risk categories separately, including:

- Difficult material
- Thin walls
- Distortion
- Interrupted cuts
- Deep pockets
- Small tools
- Long-reach tooling
- Tight tolerances
- Fine surface finish
- Complex GD&T
- Multiple datums
- Difficult workholding
- High setup count
- Expensive raw stock
- Low material availability
- Unproven process
- New customer
- New machine
- New outside vendor
- Destructive testing
- High documentation load
- Aggressive delivery schedule
- Unclear drawing
- Conflicting requirements
- Revision uncertainty
- High scrap consequence
- Low-confidence feature recognition
- Low-confidence setup proposal
- Mesh-only input
- Geometry repair
- Unit ambiguity
- Unsupported model entities
- Unknown threads or cosmetic geometry
- Tool-access uncertainty

Each risk must have:

- Category
- Description
- Probability
- Cost or time impact
- Mitigation
- Owner
- Effect on cost, price, lead time, or bid/no-bid
- Customer visibility
- Acceptance state
- Resolution state

---

## 23. Inspection and Quality

Include estimateable costs for:

- Drawing review
- Contract review
- Model-versus-drawing reconciliation
- Ballooning
- Inspection planning
- In-process inspection
- Final inspection
- CMM programming
- CMM setup
- CMM runtime
- Manual inspection
- First-article inspection
- Partial first article
- Sampling plans
- Capability studies
- Gage acquisition
- Custom gages
- Calibration considerations
- Material certifications
- Process certifications
- Certificate of conformance
- Serialization
- Traceability
- Inspection reports
- Source inspection
- Customer portals
- Record retention
- Nonconformance risk
- Destructive-test pieces
- Documentation packages

Quality requirements must influence both cost and workflow.

---

## 24. Outside Processing

Model:

- Vendor
- Approved-vendor status
- Process
- Specification
- Quote reference
- Price
- Minimum lot charge
- Setup charge
- Certification charge
- Freight in both directions
- Packaging requirements
- Lead time
- Expediting
- Yield or scrap risk
- Vendor markup
- Quote expiration
- Alternate vendors
- Customer-mandated vendors
- Approval status

---

## 25. Economics and Pricing

Separate cost from price.

Internal cost categories should include:

- Raw material
- Purchased components
- Direct labor
- Machine cost
- Programming
- Setup
- Prove-out
- Tooling
- Consumables
- Fixtures
- Inspection
- Quality documentation
- Outside processing
- Freight
- Packaging
- Non-recurring engineering
- Administrative effort
- Overhead
- Risk reserve
- Expected scrap
- Expected rework

Pricing should support:

- Markup
- Gross-margin targets
- Price floors
- Minimum order value
- Minimum line-item value
- Setup charges
- Non-recurring charges
- Quantity breaks
- Customer-specific pricing
- Strategic pricing adjustments
- Expedite charges
- Discounts
- Manual price overrides
- Approval thresholds

Explicitly distinguish markup from margin in the domain model and UI.

Use fixed-precision decimal arithmetic for money. Do not use binary floating point for authoritative currency calculations.

All units must be explicit.

---

## 26. Historical Actuals and Calibration

Plan to capture:

- Actual material usage
- Actual material price
- Actual programming time
- Actual setup time
- Actual cutting time
- Actual non-cutting time
- Actual cycle time
- Actual labor
- Actual inspection time
- Actual tooling cost
- Actual scrap
- Actual rework
- Actual outside-process cost
- Actual freight
- Actual lead time
- Actual machine used
- Actual tools used
- Actual feeds and speeds
- Actual setup count
- Job outcome
- Variance reason codes

Historical calibration must:

- Preserve the original estimate
- Preserve geometry, feature, runtime, rate, and algorithm versions
- Compare estimate versus actual by cost category
- Identify consistent bias
- Suggest changes
- Require authorization before changing approved defaults
- Avoid allowing one anomalous job to rewrite broad assumptions
- Distinguish geometry-analysis error from process-execution variance

---

## 27. Confidence Model

Develop an explainable confidence model.

Confidence should account for:

- File format
- Solid versus mesh
- Geometry validity
- Unit certainty
- Healing actions
- Feature-recognition certainty
- Tool-access certainty
- Setup-plan certainty
- Stock-selection certainty
- Runtime-estimation method
- Availability of a drawing
- Drawing completeness
- Historical similarity
- Material-price freshness
- Outside-vendor quote freshness
- Machine familiarity
- Process familiarity
- Tolerance difficulty
- Inspection certainty
- Number of unresolved assumptions

Confidence must not be presented as false mathematical certainty.

Display the reasons behind it.

---

## 28. Security and Deployment

Plan at least three deployment profiles.

### Standalone

- One workstation
- Local database
- Local files
- Offline capable
- Manual backups
- Suitable for early development and small-shop use

### Team or LAN

- Multiple authorized users
- Shared service
- Central database
- Central document storage
- Role-based access
- Audit logging
- Controlled backups
- No required public cloud dependency

### Controlled-Data Environment

- Explicit classifications
- Restricted users
- Strong authentication
- Project-based access
- Audit events
- Controlled export
- Controlled printing
- Backup policy
- Data-retention policy
- No telemetry by default
- No automatic external AI upload
- Security configuration documentation
- Deployment-boundary documentation

Plan for:

- Authentication
- Authorization
- Audit logs
- Session handling
- Secret storage
- Encryption decisions
- Backup and restore
- Attachment integrity
- Model hashing
- Revision history
- Safe previews
- Import validation
- Least-privilege filesystem access
- Dependency review
- Secure update delivery
- Sensitive-data-safe logging

---

## 29. Architecture Evaluation

The primary candidate architecture is:

- Cargo workspace
- Rust domain and application services
- Rust estimation engine
- Rust geometry-analysis abstraction
- Rust feature-recognition engine
- Rust feeds-and-speeds engine
- Rust model-viewer integration
- SQLite for standalone mode
- Repository abstraction for future team deployment
- Fixed-precision decimal types
- Serde-based structured import and export
- Cross-platform CI for Windows, Linux, and macOS

For the desktop UI, evaluate:

1. Tauri 2 with Leptos
2. Dioxus Desktop
3. Slint
4. Iced
5. egui only if it remains credible for this application

Evaluate:

- Custom visual styling
- High-density forms
- Complex tables
- Keyboard navigation
- Accessibility
- Rendering consistency
- HiDPI
- Native menus and file dialogs
- Custom title bars
- Drag and drop
- Printing
- PDF preview
- 3D model-viewer integration
- GPU rendering integration
- Startup time
- Memory use
- Packaging
- Signing
- Automated testing
- Maintainability
- Rust purity
- Licensing
- Security model
- Developer productivity

For geometry and rendering, evaluate:

- Exact B-rep kernel options
- Mesh-processing libraries
- Rust-native versus FFI approaches
- Open-source versus commercial translators
- Cross-platform packaging
- Threading and memory safety
- Geometry healing
- STEP coverage
- IGES coverage
- STL and 3MF coverage
- Rendering interoperability
- Headless testability
- Licensing implications

Create ADRs for:

- UI framework
- Geometry kernel
- Model rendering
- File-format support
- Geometry-engine process boundary
- Persistence
- Calculation versioning
- Model-analysis versioning

Do not select a framework or kernel only because its first demo is easy.

---

## 30. Proposed Module Boundaries

Evaluate a Cargo workspace resembling:

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

The geometry, runtime, and estimation engines must not depend on the UI framework.

The domain and calculation crates must be testable without launching a graphical application.

The persistence layer must be accessed through explicit repositories or application services rather than arbitrary SQL from UI components.

Consider whether risky native geometry parsing should run in a separate worker process. Document the decision.

---

## 31. Estimation Engine Requirements

Design the engine as a deterministic calculation graph or similarly traceable structure.

Every calculated result should expose:

- Value
- Unit
- Source inputs
- Formula or rule identifier
- Intermediate values
- Rounding behavior
- Rate-card version
- Feeds-and-speeds version
- Geometry-analysis version
- Feature-recognition version
- Runtime-estimation version
- Whether the value was calculated, imported, suggested, historical, or overridden
- Override reason
- Warning messages
- Confidence information

Prevent circular calculation dependencies.

Define rounding boundaries explicitly.

Preserve appropriate internal precision and round only at documented business or presentation boundaries.

---

## 32. Worked Estimate Examples

Create complete worked examples for:

- Simple three-axis aluminum part from STEP
- Tight-tolerance stainless part with drawing-driven requirements
- Multi-operation turned part
- Live-tool lathe part
- Mill-turn part
- Swiss production part
- One-off repair part from incomplete geometry
- Expensive aerospace material
- Part requiring heat treatment and coating
- Part requiring FAI and CMM inspection
- Repeat order with historical actuals
- Multiple quantity breaks
- High scrap-risk part
- Mesh-only STL estimate
- Model with unit ambiguity
- Model with geometry-repair warnings
- Part with low-confidence undercuts
- Part with competing three-axis and five-axis approaches

Each example must show:

- Imported model facts
- Detected features
- Stock choice
- Proposed setups
- Proposed tools
- Feeds and speeds
- Cutting time
- Non-cutting time
- Programming
- Setup
- Inspection
- Outside processing
- Risk
- Total cost
- Selling price
- Confidence
- Human corrections

---

## 33. UI and Experience Planning

Create a bespoke design system rather than styling screens independently.

Define:

- Typography
- Type scale
- Spacing
- Corner radii
- Elevation
- Borders
- Surfaces
- Semantic colors
- Selection states
- Focus states
- Warning states
- Error states
- Success states
- Confidence states
- Density modes
- Motion
- Iconography
- Data visualization
- Table behavior
- Form behavior
- Empty states
- Loading states
- Error recovery
- Dark and light themes

Do not depend only on color to convey meaning.

The interface should favor:

- Custom restrained application frame
- Clear workspace hierarchy
- Main 3D viewport
- Model tree
- Feature list
- Contextual inspector
- Persistent estimate totals
- Expandable cost breakdowns
- Inline editing
- Keyboard navigation
- Command palette
- Search
- Recent items
- Saved views
- Adjustable panel sizes
- Context menus
- Undo and redo
- Autosave
- Visible analysis status
- Minimal modal dialogs
- Progressive disclosure
- Batch entry
- Templates
- Clear override indicators
- Immediate recalculation feedback

The UI must provide visual mapping between:

- Selected model geometry
- Detected features
- Setups
- Tools
- Routing operations
- Cost contribution
- Confidence warnings

Plan screens for:

- Home or command center
- RFQ inbox
- Quote list
- Quote workspace
- Part workspace
- Model import
- Model viewer
- Geometry validation
- Feature review
- Setup and orientation review
- Stock selection
- Routing editor
- Tool and feeds/speeds review
- Material selector
- Workcenter selector
- Operation editor
- Cost breakdown
- Quantity comparison
- Risk review
- Approval review
- Customer quote preview
- Material library
- Tool library
- Machine library
- Rate-card management
- Outside-vendor library
- Customer management
- Historical actuals
- Variance analytics
- Templates
- User administration
- Backup and restore

---

## 34. First Vertical Slice

The first production vertical slice must prove both the desktop architecture and CAD-assisted estimating.

It should allow a user to:

1. Create an RFQ.
2. Add a customer.
3. Add one part and revision.
4. Import a STEP, STL, or 3MF model.
5. Confirm model units.
6. View the model.
7. See geometry warnings.
8. See bounding dimensions.
9. See volume and surface area.
10. Select a material.
11. Calculate part mass.
12. Receive a suggested rectangular or round stock size.
13. See removed material volume.
14. Choose a broad process class such as milling or turning.
15. Receive a rough setup-count suggestion.
16. Receive a coarse runtime estimate based on material removal, geometry complexity, and configurable baseline feeds and speeds.
17. Edit the proposed setup, runtime, machine, and material assumptions.
18. Add programming, inspection, tooling, outside-process, and risk values.
19. Create quantity breaks.
20. Calculate internal cost.
21. Apply pricing rules.
22. View a transparent calculation breakdown.
23. Record assumptions and overrides.
24. Save and reopen the estimate.
25. Preserve the source model hash and analysis version.
26. Preview an internal estimate summary.
27. Preview a customer-facing quote.
28. Run on Windows, Linux, and macOS.

The first slice does not require full automatic pocket, hole, or five-axis feature recognition, but its architecture must support later feature-based analysis.

---

## 35. Deliberate Non-Goals for Initial Releases

Evaluate and document whether these should be deferred:

- Production-safe G-code generation
- Full CAM replacement
- Perfect feature recognition
- Guaranteed setup planning
- Automatic tolerance extraction from all drawings
- Native import of every proprietary CAD format
- ERP replacement
- Full production scheduling
- Machine monitoring
- Automatic regulatory-compliance certification
- Unreviewed AI-generated quotes
- Public instant quoting
- Mobile application
- Cloud-only deployment
- Automatic submission of technical data to external AI systems

---

## 36. Roadmap

Create a phased roadmap similar to:

### Phase 0 — Discovery and Validation

- Interview estimators, programmers, machinists, quality personnel, and purchasing
- Collect anonymized quote examples
- Collect representative 3D models
- Identify shop rates
- Identify common machines and tools
- Identify common materials
- Validate cost categories
- Establish security boundaries
- Complete UI and geometry technical spikes

### Phase 1 — Foundation

- Cargo workspace
- Domain types
- Units
- Fixed-precision money
- Geometry abstraction
- STEP/STL/3MF import spike
- Model-viewer spike
- Basic geometry measurements
- Estimation engine foundation
- Persistence foundation
- Design system
- Desktop shell
- CI matrix
- Sample model fixtures

### Phase 2 — CAD-Assisted Core Vertical Slice

- RFQs
- Customers
- Parts
- Model import
- Model viewing
- Units and validation
- Bounding box
- Volume
- Surface area
- Material mass
- Stock suggestion
- Removed volume
- Coarse process selection
- Rough setup suggestion
- Rough runtime estimate
- Editable routing
- Quantity breaks
- Cost calculation
- Pricing
- Save and reopen
- Internal summary
- Customer quote preview

### Phase 3 — Feature-Based Estimation

- Hole recognition
- Pocket recognition
- Slot recognition
- Turning-profile recognition
- Better stock selection
- Tool assignment
- Feature-based runtime
- Setup orientation analysis
- Better confidence scoring

### Phase 4 — Shop Libraries

- Material library
- Tool library
- Machine library
- Workcenter rates
- Operation templates
- Outside vendors
- Inspection templates
- Quote templates
- Versioned rate cards
- Versioned feeds and speeds

### Phase 5 — Aerospace and Quality Depth

- Requirement checklists
- FAI estimating
- CMM estimating
- Certifications
- Traceability
- Source inspection
- Controlled attachments
- Approval workflows

### Phase 6 — Actuals and Calibration

- Record actuals
- Variance analysis
- Historical comparisons
- Similar-job references
- Controlled recommendations
- Machine- and tool-specific calibration

### Phase 7 — Advanced Geometry and Process Planning

- Better undercut analysis
- Five-axis accessibility
- Mill-turn planning
- Swiss planning
- Simplified toolpath generation
- Machine-power validation
- Tool-life modeling
- Drawing assistance
- Similarity search

### Phase 8 — Team Deployment

- Multi-user service
- Central storage
- Authentication
- Role-based access
- Audit logging
- Backup administration
- Conflict strategy

Refine the roadmap based on research and shop needs.

---

## 37. Testing Strategy

Plan tests at several levels.

### Unit tests

- Money arithmetic
- Unit conversions
- Material mass
- Stock dimensions
- Removed volume
- Setup amortization
- Quantity breaks
- Margin and markup
- Scrap calculations
- Routing totals
- Runtime calculations
- Lead-time calculations
- Rounding

### Geometry golden tests

For each sample model, preserve expected:

- Units
- Bounding dimensions
- Volume
- Surface area
- Body count
- Validity state
- Stock suggestion
- Warning set

### Feature-recognition tests

For known fixtures, preserve expected:

- Feature count
- Feature types
- Dimensions
- Confidence
- Known ambiguous regions

### Runtime golden tests

Store reviewed model-to-runtime examples with:

- Material
- Machine
- Tools
- Feeds
- Speeds
- Setup assumptions
- Cutting time
- Non-cutting time
- Total cycle time

### Property tests

Examples:

- Material mass cannot be negative.
- Removed volume cannot be negative when stock fully encloses the model.
- Make quantity cannot be below deliver quantity.
- Total cost must not decrease when a positive cost component increases unless a documented nonlinear rule applies.
- Selling price must not fall below a configured floor without override.
- Unit conversions should round-trip within documented tolerances.
- Re-analysis must not overwrite an approved manual routing.
- Mesh confidence must not exceed documented limits when exact topology is unavailable.

### Persistence tests

- Migrations
- Save and reopen
- Model hash persistence
- Analysis snapshot persistence
- Revision history
- Backup and restore
- Corruption handling

### UI tests

- Model import
- Unit confirmation
- Model selection
- Feature selection
- Setup editing
- Routing editing
- Cost drill-down
- Keyboard navigation
- Focus behavior
- Validation
- Cross-platform rendering
- High-DPI behavior

### Security tests

- Permission enforcement
- Unauthorized file access
- Audit events
- Path traversal
- Malformed imports
- Sensitive logging
- Backup permissions
- Update integrity
- Parser-failure containment

---

## 38. Planning Questions

Identify questions requiring shop input, including:

- Which machines and workcenters are currently used?
- Which operations are done in-house?
- Which are outsourced?
- Which CAD formats are most common?
- Are assemblies quoted, or mostly single parts?
- Are STEP files generally available?
- How often are STL or mesh files received?
- What model-unit errors occur in practice?
- Which materials are most common?
- Which cutting-tool brands and families are common?
- Are shop-specific feeds and speeds documented?
- How are machine and labor rates calculated?
- Are setup and run rates different?
- How is overhead allocated?
- How are margin and markup used?
- How are scrap and risk handled?
- What historical quote and job data exists?
- Which ERP, CAM, QMS, or time-tracking systems exist?
- What quality clauses appear most often?
- Who may view defense-related models?
- Is offline or air-gapped operation required?
- Is multi-user operation required for the first release?
- What quote output format is currently used?
- Who approves pricing?
- How are quote revisions controlled?
- What actual job data can be fed back into the estimator?
- Which parts should become validation fixtures?
- What estimation accuracy is acceptable for the first release?
- How should confidence be displayed?
- Which operations should never be automatically inferred?

Do not stop all planning because some answers are unavailable. Record assumptions and create validation tasks.

---

## 39. Planning Completion Criteria

The planning phase is ready for implementation only when:

- The documentation index exists and is navigable.
- The project-state document is current.
- Core terminology is defined.
- Major user workflows are documented.
- Functional and nonfunctional requirements have stable IDs.
- CAD format support has an explicit matrix.
- Geometry-kernel evaluation is complete.
- UI framework evaluation is complete.
- Model-viewer evaluation is complete.
- The initial domain model is documented.
- Geometry-analysis stages are documented.
- Runtime-estimation levels are documented.
- Feeds-and-speeds versioning is documented.
- Calculation boundaries are documented.
- Worked estimate examples exist.
- Representative model fixtures exist.
- The initial data model is documented.
- Security boundaries are documented.
- Deployment modes are documented.
- Required ADRs are approved or ready for review.
- The first CAD-assisted vertical slice has acceptance criteria.
- The testing strategy is defined.
- The roadmap and backlog exist.
- Major risks are recorded.
- Open shop-specific questions are listed.
- The first implementation task is unambiguous.

---

## 40. Required Final Planning Report

At the conclusion of this prompt, provide:

1. A summary of files created or changed.
2. The proposed product architecture.
3. The recommended UI framework and reasons.
4. The recommended geometry kernel and reasons.
5. The recommended model-rendering approach.
6. The proposed file-format support matrix.
7. The proposed initial domain model.
8. The geometry-processing pipeline.
9. The feature-recognition roadmap.
10. The runtime-estimation structure.
11. The feeds-and-speeds strategy.
12. The first CAD-assisted vertical slice.
13. The milestone roadmap.
14. The highest-risk assumptions.
15. Questions requiring shop input.
16. The exact first development task.
17. Build or prototype commands currently available.
18. A planning-readiness verdict.

Before finishing:

- Update `docs/INDEX.md`.
- Update `docs/PROJECT_STATE.md`.
- Update the progress log.
- Record all unresolved architecture decisions.
- Confirm that future agents are directed to follow the documentation system.

All future planning and implementation must remain aligned with this documentation.
