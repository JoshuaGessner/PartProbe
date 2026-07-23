# Actuals and Calibration

> **Status:** Draft  
> **Last updated:** 2026-07-22  
> **Related requirements:** REQ-F-012, REQ-F-056–REQ-F-059; CALC-020; DATA-012, DATA-031–DATA-034  
> **Related ADRs:** ADR-0007, ADR-0008, ADR-0013, ADR-0014  
> **Open questions:** OQ-019, OQ-020, OQ-027, OQ-044–OQ-046  
> **Dependencies:** Historical-data availability and authorization policy  
> **Supersedes:** None

Actuals capture material usage/price, programming, setup, cutting/non-cutting/cycle time, labor, inspection, tooling, scrap/rework, outside process, freight, lead time, machine/tools/feeds/speeds/setups, job outcome, and variance reason codes.

The feedback chain is `estimate → reviewed CAM plan → machine result → final actual`, with incomplete stages allowed and visibly unmapped. Comparison is category/operation-specific and preserves exact estimate, routing, source, CAM import, machine/actual and algorithm/library/rate versions. Variance distinguishes geometry, feature, setup, tool, feeds/speeds, runtime, programmer, machine/operator, inspection, material, rework, vendor, and schedule causes.

Estimator corrections are useful earlier evidence but are not production actuals. Correction and actual cohorts remain distinguishable, access controlled, and segmented by process/material/machine/customer evidence. Calibration may propose a versioned change; approval, comparison and rollback are mandatory, and one anomaly, one estimator, or one customer pattern cannot silently rewrite defaults.
