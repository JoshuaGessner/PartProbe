# CAD file-format research

## Metadata

- **Status:** In Review
- **Last updated:** 2026-07-22
- **Related requirement IDs:** REQ-F-021, REQ-F-022, REQ-NF-014, GEO-001–GEO-009, SEC-004
- **Related architecture decision IDs:** ADR-0002, ADR-0004, ADR-0005
- **Open questions:** What formats and maximum file sizes occur in shop RFQs? Is assembly quoting needed in the first release? Can commercial translators be funded later?
- **Dependencies:** Geometry-kernel spike; import-worker sandbox; fixture corpus; `geometry-analysis-model.md`
- **Supersedes / superseded by:** None / none

## Decision-oriented conclusion

Propose **STEP AP203/AP214/AP242, STL, and 3MF** for the first vertical slice, pending ADR-0004 evidence and approval. Treat STEP as the sole proposed first-class exact-engineering interchange input; STL and 3MF are mesh inputs whose measurements and inferred features carry a hard confidence ceiling. Propose **IGES and OBJ only as preview/review secondary inputs** after the primary path is proven. Do not promise native/proprietary formats, Parasolid XT, or ACIS/SAT: each needs a separately purchased, redistributable translator and a packaging proof.

The proposed Open CASCADE Technology (OCCT) spike is credible because its data-exchange documentation names STEP (AP203, AP214, AP242), IGES, and STL interfaces and describes validity checks; it does not make every imported model trustworthy. [OCCT data exchange](https://dev.opencascade.org/doc/overview/html/index.html) [OCCT STEP translator](https://dev.opencascade.org/doc/overview/html/occt_user_guides__step.html)

## Support matrix

| Format | Geometry and units | Useful metadata / assembly | Exact analysis quality | Initial disposition | Principal limitations and import route |
|---|---|---|---|---|---|
| STEP `.stp/.step` | Exact B-rep surfaces, curves, topology; AP schema can declare units | AP242/AP214 can carry product names, colors and assemblies | High when translated solid validates | **First-class** | OCCT XDE import; preserve source + transfer report. Do not infer PMI, tolerances, threads, or material requirements as authoritative. |
| STL `.stl` | Triangle mesh; **no standardized unit field** | No assembly, reliable material, color, layer, or design history | Low–medium, mesh-derived only | **First-class** | Parse with explicit size/triangle limits; require unit confirmation unless package context is trustworthy. Report watertight/manifold/normal state. |
| 3MF `.3mf` | ZIP/OPC package containing triangle meshes; root model unit is explicit (default millimeter) | Build items, component composition, transforms, metadata; extensions can convey properties | Medium, mesh-derived only | **First-class** | Validate ZIP paths and XML/resource limits; honor only supported required extensions. 3MF is not a substitute for a CAD B-rep. |
| IGES `.igs/.iges` | Curves, surfaces, and some B-rep; unit declarations exist but translation varies | Names/colors available in some files; weak, legacy assembly semantics | Medium–low | **Secondary** | OCCT reader / Shape Healing; accept converted solids only after validation, retain entity-level transfer warnings. OCCT notes only 3D geometric entities are translated. [IGES guide](https://dev.opencascade.org/doc/overview/html/occt_user_guides__iges.html) |
| OBJ `.obj` + `.mtl` | Mostly polygon mesh; formal unit semantics absent; optional freeform surfaces poorly portable | Groups, material references, textures; no structured embedded metadata | Low, mesh-derived | **Secondary** | Parse OBJ with explicitly selected companion assets; never follow arbitrary external texture paths. The Library of Congress notes its lack of structured metadata and external MTL/texture dependency. [LOC format assessment](https://www.loc.gov/preservation/digital/formats/fdd/fdd000507.shtml) |
| Parasolid XT `.x_t/.x_b` | Exact native Parasolid B-rep / facets | Kernel-specific metadata, assembly depends on SDK | Potentially high | **Future / licensed** | Siemens SDK evaluation, license/redistribution and FFI worker proof required; no public commitment. [Siemens SDK](https://www.siemens.com/en-us/products/plm-components/parasolid/3d-modeling-sdk/) |
| ACIS/SAT `.sat/.sab` | Exact ACIS B-rep | Kernel-specific attributes / assemblies | Potentially high | **Future / licensed** | Spatial SDK/translator agreement and deployment proof required. [Spatial ACIS](https://www.spatial.com/solutions/3d-modeling/3d-acis-modeler) |
| SolidWorks, Inventor, Creo, CATIA, NX, Fusion native | Proprietary, version-dependent | Potentially rich | Unknown until licensed translator test | **Conversion only** | Ask sender for STEP AP242 plus drawing; later evaluate commercial translation per vendor/version/OS. Native names must never be promised from extension sniffing alone. |

### Format evidence and handling rules

- 3MF’s current published core specification defines a model `unit` with values from micron through meter and says its default is millimeter; it also defines document metadata and component composition. [3MF Core Specification](https://github.com/3MFConsortium/spec_core/blob/master/3MF%20Core%20Specification.md) The consortium states necessary patents for public specifications are royalty-free, but the chosen implementation library still needs an independent license review. [3MF specification page](https://3mf.io/spec/)
- OCCT’s STEP XDE reader documents transfer of assemblies, names, colors, layers and validation properties. Preserve transferred attributes as provenance, but never map colors/layers to manufacturing requirements without user acceptance. [OCCT STEP translator](https://dev.opencascade.org/doc/overview/html/occt_user_guides__step.html)
- An extension is evidence only. Identify the format by safe content inspection, not filename, and reject extension/content mismatches with a recoverable error.

## Common failure modes and response

| Condition | Response | Confidence / audit effect |
|---|---|---|
| Unknown or conflicting units; implausible scale | Show dimensions in alternative common units and require confirmation; never silently rescale | Block approval; record candidate scales and human decision |
| STEP transfer has failed entities or invalid shells | Retain original, show transfer/healing report, permit preview only if safe; do not use exact volume | Cap geometry and feature confidence; create risk |
| Mesh has holes, self-intersections, zero-area triangles, non-manifold edges, inconsistent normals | Report counts and locations; optionally make a non-destructive display repair, never replace source | No mass/volume claims unless closed, oriented, validated mesh; cap at low |
| Assembly / multiple bodies | Analyze per body and document transforms; no automatic combined stock/routing | Require scope choice; assembly gets an explicit warning |
| Excessive compressed package ratio, XML nesting, triangle/entity count | Enforce quotas in worker; terminate and diagnose | Import fails safely; no model contents in logs |
| Missing OBJ companion assets | Geometry can preview; appearance warning; ignore paths outside controlled import package | No estimator impact unless user relies on color annotations |

## Packaging, licensing, and security gates

1. Ship OCCT only after a legal review of its LGPL-2.1-with-additional-exception notice, relinking/replacement obligations, dynamic/shared-library deployment per target, and third-party notices. OCCT itself calls out LGPL section 6 obligations for proprietary applications. [OCCT licensing](https://dev.opencascade.org/resources/licensing)
2. Keep CAD reader/kernel code and its native libraries in a versioned worker distribution. Record library build, translator configuration, OS/CPU and importer algorithm version in each analysis snapshot.
3. Open untrusted CAD packages under a quota-enforced worker with no network, a dedicated temp directory, allow-listed input path, timeout, memory/CPU limits and sanitized diagnostics. Never resolve external references by default.
4. Before a commercial format is enabled, obtain written terms covering supported formats/versions, offline deployment, redistribution, seat/concurrency accounting, updates, export restrictions, support, security notices, and cost. License availability is not format support.

## Research limitations and validation needed

This is a design recommendation, not a representation-quality guarantee. STEP conformance, translation behavior, and whether a supplier’s CAD export contains usable B-rep must be measured against real RFQ fixtures. The format spike must publish a per-format fixture result, model hash, transfer/healing report, calculation tolerance, peak worker resources, and package artifact list before ADR-0004 can be approved.
