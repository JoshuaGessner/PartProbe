# CAD Import Requirements

> **Status:** In Review  
> **Last updated:** 2026-08-28
> **Related requirements:** REQ-F-002–REQ-F-004; REQ-NF-002, REQ-NF-005; GEO-001–GEO-007  
> **Related ADRs:** ADR-0002, ADR-0004, ADR-0005  
> **Open questions:** OQ-003–OQ-008  
> **Dependencies:** Kernel/import spikes  
> **Supersedes:** None

| ID | Rule |
|---|---|
| GEO-001 | Intake computes SHA-256 (or approved successor), records byte size and claimed/detected format before parsing, and never modifies source bytes. |
| GEO-002 | Format detection uses content/signature where feasible; extension mismatch is a warning. |
| GEO-003 | Extracted units include source and confidence. Missing/conflicting units require user confirmation before dimension-dependent calculations. |
| GEO-004 | STEP produces B-rep evidence only for successfully translated representation; STL, 3MF, and OBJ mesh evidence shall never be labeled exact solid topology. 3MF preserves per-object/build-item unit, transform, indexed mesh provenance, and Core index-defined adjacency; distinct vertex indices are not welded by coordinate equality. STL has no index authority and uses its separately versioned welding/tolerance policy. Both mesh paths use the mesh confidence ceiling. |
| GEO-005 | Healing is optional, versioned, reproducible where feasible, and records each action plus pre/post validity. |
| GEO-006 | Parser failure is recoverable and creates sanitized diagnostics; no geometry coordinates/names enter logs by default. |
| GEO-007 | Assemblies, multi-body files, unsupported entities, colors/layers, and metadata are reported rather than silently flattened. |

First class: STEP, STL, 3MF. Experimental/secondary: IGES and OBJ. Proprietary/native formats require conversion until a licensed translator ADR says otherwise.
