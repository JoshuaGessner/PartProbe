# Routing and Operation Model

> **Status:** In Review  
> **Last updated:** 2026-07-22  
> **Related requirements:** REQ-F-006–REQ-F-009; DATA-007  
> **Related ADRs:** ADR-0007, ADR-0008  
> **Open questions:** OQ-002, OQ-030  
> **Dependencies:** Setup, runtime, quality, outside-process models  
> **Supersedes:** None

A routing is an ordered, revisioned proposal. Operations cover procurement, blanking, machining, EDM/grinding/manual work, inspection, special/outside processing, assembly, marking, packaging, shipping, and administration.

Each operation records source, workcenter/vendor, setup/orientation, assigned features, tool assemblies, setup/program/prove-out/first-piece/cutting/non-cutting/load times, batch and calendar assumptions, labor/machine/unattended allocation, inspection frequency, scrap/rework, tooling/consumables/fixtures, charges/freight, notes, assumptions, confidence, and override chain.

Re-analysis creates a comparison against the active routing. Accepted human operations persist until explicitly replaced; automatic proposals cannot overwrite them.
