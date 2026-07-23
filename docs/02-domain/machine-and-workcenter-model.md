# Machine and Workcenter Model

> **Status:** Draft  
> **Last updated:** 2026-07-22  
> **Related requirements:** REQ-F-006–REQ-F-008; DATA-006  
> **Related ADRs:** ADR-0006  
> **Open questions:** OQ-001, OQ-012, OQ-013  
> **Dependencies:** Shop machine and rate data  
> **Supersedes:** None

`MachineCapability` is separated from `WorkcenterRate`. Capability captures envelope, axes, spindle limits/torque evidence, feeds/rapids, tool/bar capacity, rotary/accuracy/process capability, restrictions, and default time parameters. Rates capture effective period, setup/run labor, machine/burden components, unattended policy, minimums, cost center, and approval.

Supported classifications include 3/4/5-axis mills; CNC/live-tool/mill-turn/Swiss/manual lathes; manual mills; saw, grinder, wire/sinker EDM, router, inspection/CMM, deburr, clean, and packaging workcenters. Configuration—not hard-coded enums—holds shop-specific inventory.
