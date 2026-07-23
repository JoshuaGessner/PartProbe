# Runtime Estimation Requirements

> **Status:** In Review  
> **Last updated:** 2026-07-22  
> **Related requirements:** TIME-001–TIME-008; REQ-F-006–REQ-F-008, REQ-NF-003  
> **Related ADRs:** ADR-0007, ADR-0008  
> **Open questions:** OQ-001, OQ-010–OQ-013, OQ-020, OQ-027  
> **Dependencies:** Machine/tool/feed-speed libraries and actuals  
> **Supersedes:** None

| ID | Requirement |
|---|---|
| TIME-001 | Every runtime result shall identify method: coarse volumetric, complexity-adjusted, feature, simplified path, imported CAM/NC, or historical comparison. |
| TIME-002 | Cutting, rapid/positioning, tool change, spindle, probing, indexing/transfer, load/unload, inspection, and operator events shall remain separately traceable. |
| TIME-003 | Tool/material/machine/feed/speed/engagement/power/torque/tool-life inputs shall be versioned, sourced, constrained, and warning-producing. |
| TIME-004 | Missing evidence shall reduce method/confidence or block a term; it shall not silently become zero or universal overhead. |
| TIME-005 | Setup, programming, prove-out, cycle, lot, queue, outside-process lead, attended, and elapsed times shall use distinct units/bases. |
| TIME-006 | Runtime proposals shall expose formula/intermediates, scope, assumptions, confidence reasons, and override history. |
| TIME-007 | Validation shall segment error/bias by method, process, material, machine, quantity, and scope change; no single accuracy claim applies universally. |
| TIME-008 | Runtime output shall never be presented as production CAM simulation or safe machine-cycle guarantee. |
