# Tooling Model

> **Status:** Draft  
> **Last updated:** 2026-07-22  
> **Related requirements:** REQ-F-006, REQ-F-007; TIME-003  
> **Related ADRs:** ADR-0008  
> **Open questions:** OQ-010, OQ-011  
> **Dependencies:** Shop tool catalog  
> **Supersedes:** None

Separate `ToolFamily`, `CuttingTool`, `Insert`, `Holder`, and `ToolAssembly`. A versioned assembly records effective diameter, reach, flute length, insert geometry, material/coating, flute/insert count, holder, stickout, compatibility, cost basis, expected life, replacement allowance, and source.

Candidate tools are suggestions constrained by geometry access, operation, work material, machine interface/capability, and approved feed/speed records. Tool life is an assumption with evidence, never a universal constant. Custom tooling and fixture costs remain explicit NRE/recurring components.
