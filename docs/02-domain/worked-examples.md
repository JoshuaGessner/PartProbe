# Worked Estimate Examples

> **Status:** Draft  
> **Last updated:** 2026-07-22  
> **Related requirements:** REQ-F-002–REQ-F-010; CALC-001–CALC-020; TEST-002, TEST-005  
> **Related ADRs:** ADR-0002–ADR-0008  
> **Open questions:** OQ-009–OQ-20, OQ-027  
> **Dependencies:** Shop review and executable calculation fixtures  
> **Supersedes:** None

These synthetic examples force domain coverage; they are **not approved shop standards**. Currency is USD, times are lot hours unless `/part` is shown, and feeds/speeds are deliberately broad profiles rather than unsafe universal cutting data. Every example assumes one deliverable part unless quantity is stated. Totals illustrate the complete result shape and must be replaced by executable, reviewed golden fixtures during TASK-001/TASK-002.

| # | Scenario and imported facts | Detected/model facts | Stock, process, setups, tools, feed/speed source |
|---|---|---|---|
| EX-01 | STEP, mm, valid solid; 100×70×25 mm Al 6061 part | 1 body; 175 cm³; planar faces, 2 pockets, 6 hole candidates | 105×75×30 mm plate; 3-axis mill; 2 setups; face/end mills + drills; conservative baseline v0 |
| EX-02 | STEP + drawing; 80×45×20 mm 17-4PH H900 | Valid solid; tight bores and thin web; tolerances drawing-derived | 85×50×25 mm bar; 3-axis mill + grind; 3 setups; carbide tools; reviewed stainless profile |
| EX-03 | STEP, mm; rotational 60×140 mm 4140 | Revolved envelope; OD steps, bore, groove, thread candidate | Ø65×150 bar; CNC lathe; 2 ops; CNMG/groove/drill/thread tools; 4140 turning profile |
| EX-04 | STEP + drawing; Ø50×90 mm 303 SS | Rotational body plus cross holes/flats | Ø55 bar; live-tool lathe; 2 setups/transfer; turning tools + live drill/end mill; 303 profile |
| EX-05 | STEP; 120×85×75 mm 15-5PH | Turned journals plus non-axisymmetric pockets | Ø130 billet; mill-turn alternative; 3 setups; turning, milling, probing; conservative approved profiles |
| EX-06 | STEP; Ø12×80 mm 316L, qty 2,000 | Slender revolved part, cross hole, thread candidate | Ø12.7 bar; Swiss; 1 bar-fed program; micro tools; supplier/tool-vendor data adjusted by shop |
| EX-07 | STL, unit unknown; repair bracket | Watertight mesh after repair; approximate 140×90×35 mm; no exact topology | Customer-supplied damaged part + billet; manual/CNC hybrid; 4 setups; manual selection; assumed units |
| EX-08 | STEP + cert requirement; 160×100×45 mm Ti-6Al-4V | Valid solid; deep pockets; high removed volume | 170×110×55 certified plate; 5-axis positional; 3 setups; high-performance carbide; conservative Ti profile |
| EX-09 | STEP + drawing; 90×60×30 mm 4340 | Valid; finish stock allowance | 95×65×35 bar; mill → heat treat → grind/coating; 3 internal setups; vendor quotes |
| EX-10 | STEP + drawing; 200×150×40 mm Al 7075 | Many bores; GD&T/FAI requirements are drawing-derived | 210×160×45 plate; 3-axis mill; 3 setups; milling/drilling/probing; approved Al profile |
| EX-11 | Prior STEP rev, repeat qty 50 | Same source hash as approved prior revision; actuals available | Same plate/routing; 2 setups; pinned tool set; approved historical calibration cohort |
| EX-12 | STEP, qty breaks 1/10/50/200 | Valid prismatic part; 1 pocket, 4 holes | Plate; 2 setups; standard tool set; same technical assumptions, independently recalculated breaks |
| EX-13 | STEP; thin magnesium housing, qty 5 | Thin walls; large envelope; high distortion/scrap consequence | Oversize plate; staged 3-axis machining; 4 setups; sharp tooling; low-engagement conservative profile |
| EX-14 | STL, mm confirmed; 75×60×18 mm | Watertight mesh; approximate volume/area; feature topology unavailable | 80×65×25 plate; user-selected milling; 2 setups; generic tools/profile; mesh cap on confidence |
| EX-15 | STL, no metadata; measured span 25.4 model units | Two plausible scales: 25.4 mm or 25.4 in | No stock until unit confirmation; candidate 3-axis; blocked automatic price |
| EX-16 | STEP with open shell and bad edges | Healing produced closed candidate; original and healed shapes retained | 110×80×35 plate; 3-axis proposal; 3 setups; generic tool set; healing warning |
| EX-17 | STEP; 100×100×80 mm tool steel | Deep lateral region suggests undercut, access uncertain | Block; 3-axis + EDM alternative; 4 setups; long-reach/EDM; low confidence |
| EX-18 | STEP; 180×140×70 mm Al 7075 | Five machining directions; deep pockets; valid solid | Competing 3-axis (6 setups) vs 5-axis (2 setups); separate tools/profiles and costs |

| # | Proposed timing: programming; setup; cutting; non-cutting | Inspection / outside processing | Risk and human correction |
|---|---|---|---|
| EX-01 | 2.0; 3.0; 0.42/part; 0.18/part | 1.0; none | Medium; estimator changes stock Z from 28 to 30 mm and records saw allowance |
| EX-02 | 4.0; 5.0; 1.8/part; 0.6/part | 3.5 + grind inspection; outside grind $180 | High tolerance risk; reviewer adds drawing-driven hone/grind operation |
| EX-03 | 2.5; 3.0; 0.55/part; 0.20/part | 1.0; none | Medium; thread candidate rejected and manual thread specification added |
| EX-04 | 3.5; 3.5; 0.70/part; 0.25/part | 1.2; none | Medium; cross-hole tool changed after programmer review |
| EX-05 | 7.0; 8.0; 3.2/part; 1.1/part | 2.5; none | High; mill-turn proposal replaced with mill + lathe alternate pending machine availability |
| EX-06 | 8.0; 6.0; 0.035/part; 0.015/part | 8.0 lot sampling; passivation $650/lot | Medium; sampling frequency raised and spare quantity added |
| EX-07 | 5.0; 7.0; 4.0; 2.0 | 2.0; weld repair $250 | Very high; estimator confirms inch scale and replaces model-derived dimensions with manual inspection |
| EX-08 | 10.0; 9.0; 6.5/part; 1.8/part | 4.0; none | High material/scrap; stock source changed to quoted certified lot |
| EX-09 | 4.0; 6.0; 2.0/part; 0.7/part | 3.0; heat treat $300 + coating $220 | High outside-process lead risk; vendor quote expiry corrected |
| EX-10 | 8.0; 7.0; 3.5/part; 1.0/part | FAI 12.0 + CMM 7.0; none | Medium; quality manager adds ballooning and source-inspection allowance |
| EX-11 | 0.8; 2.5; 0.38/part; 0.17/part | 5.0 lot; none | Low/medium; historical suggestion accepted but original estimate preserved |
| EX-12 | 2.0; 3.0 per lot; 0.40/part; 0.18/part | 0.4 first + sampling; none | Medium; estimator changes only 200-piece fixture amortization |
| EX-13 | 8.0; 10.0; 5.0/part; 1.5/part | 4.0; hazardous-material/vendor review unresolved | Very high distortion; reviewer adds stress-relief/inspection checkpoints |
| EX-14 | 3.0; 4.0; 1.0/part; 0.35/part | 1.2; none | Low geometry confidence; user accepts bounding facts but defines holes manually |
| EX-15 | Blocked; blocked; blocked; blocked | Blocked; none | Unknown; user confirms mm before stock/runtime calculation proceeds |
| EX-16 | 4.0; 5.0; 1.6/part; 0.5/part | 1.5; none | Low/medium; user approves healed body for estimate while source remains unchanged |
| EX-17 | 6.0; 8.0; 4.0/part; 1.2/part | 2.0; EDM quote $900 alternative | Low; programmer rejects long-reach mill plan and selects EDM alternative |
| EX-18 | 3-axis: 10/14/7.0/2.5; 5-axis: 12/7/4.5/1.5 | 3.0 each; none | Medium; estimator chooses 5-axis on lead time despite higher machine rate |

| # | Internal cost | Risk reserve | Selling price | Confidence rationale |
|---|---:|---:|---:|---|
| EX-01 | 485.00 | 35.00 | 702.00 | High geometry; medium runtime; no drawing |
| EX-02 | 1,920.00 | 280.00 | 2,970.00 | High geometry; low requirement/runtime certainty until review |
| EX-03 | 610.00 | 65.00 | 911.25 | High geometry; medium feature/runtime certainty |
| EX-04 | 720.00 | 80.00 | 1,080.00 | High geometry; medium live-tool timing |
| EX-05 | 3,850.00 | 700.00 | 6,142.50 | Medium process selection; machine availability unresolved |
| EX-06 | 18,400.00/lot | 1,600.00 | 27,000.00/lot | High repetition leverage; medium micro-tool life evidence |
| EX-07 | 2,300.00 | 850.00 | 4,252.50 | Low model fidelity and incomplete requirements |
| EX-08 | 8,900.00 | 1,600.00 | 14,175.00 | High geometry; medium process; high material consequence |
| EX-09 | 2,250.00 | 350.00 | 3,510.00 | High geometry; medium outside-vendor freshness |
| EX-10 | 5,600.00 | 600.00 | 8,370.00 | High model/drawing evidence; FAI scope needs customer confirmation |
| EX-11 | 6,900.00/lot | 300.00 | 9,720.00/lot | High historical similarity with preserved version context |
| EX-12 | 1: 520.00; 10: 1,650.00; 50: 6,700.00; 200: 24,000.00 | 45; 100; 280; 800 | 763; 2,363; 9,423; 33,480 | High geometry; quantity-specific commercial confidence |
| EX-13 | 7,800.00 | 2,900.00 | 14,445.00 | Low process yield confidence; no historical cohort |
| EX-14 | 690.00 | 140.00 | 1,120.50 | Mesh measurements medium; features low by policy |
| EX-15 | Not calculated | Not calculated | Not quotable | Unit ambiguity is a blocker, not a guessed confidence score |
| EX-16 | 980.00 | 260.00 | 1,674.00 | Healed result medium/low; source defect explicit |
| EX-17 | Mill: 2,900.00; EDM: 3,450.00 | 900; 450 | 5,130.00; 5,265.00 | Access low for mill; vendor evidence medium for EDM |
| EX-18 | 3-axis: 4,800.00; 5-axis: 4,500.00 | 600; 450 | 7,290.00; 6,682.50 | Geometry high; both process timings medium pending shop benchmark |

## Acceptance path

Each example becomes structured test data with exact inputs, a rule-version pin, itemized cost reconciliation, approved tolerances, and reviewer identity. Until then, no numeric value here may seed production defaults.
