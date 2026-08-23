//! Bounded, path-free 3MF package comparison spike.

use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;
use std::io::{Cursor, Read};

use partprobe_geometry_core::{
    GeometryWarningCode, ModelFormat, ModelLengthUnit, RepresentationBasis, UnitResolutionMethod,
};
use quick_xml::events::{BytesStart, Event};
use quick_xml::{Reader, XmlVersion};
use serde::Serialize;
use zip::{CompressionMethod, ZipArchive};

use crate::mesh_analysis::{
    MeshAnalysisError, MeshVector3, Triangle, analyze_triangles, mesh_warning_codes,
    validate_triangle,
};

/// Versioned algorithm identity for the governed 3MF package-policy slice.
pub const THREE_MF_ANALYZER_VERSION: &str = "partprobe-3mf-spike-v5";

const CONTENT_TYPES_PART: &str = "[Content_Types].xml";
const ROOT_RELATIONSHIPS_PART: &str = "_rels/.rels";
const CORE_NAMESPACE: &str = "http://schemas.microsoft.com/3dmanufacturing/core/2015/02";
const CONTENT_TYPES_NAMESPACE: &str =
    "http://schemas.openxmlformats.org/package/2006/content-types";
const RELATIONSHIPS_NAMESPACE: &str =
    "http://schemas.openxmlformats.org/package/2006/relationships";
const RELATIONSHIP_TYPE: &str = "http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel";
const MODEL_CONTENT_TYPE: &str = "application/vnd.ms-package.3dmanufacturing-3dmodel+xml";

/// Explicit package and geometry limits applied before and during 3MF analysis.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ThreeMfLimits {
    max_input_bytes: usize,
    max_entries: usize,
    max_total_uncompressed_bytes: u64,
    max_model_xml_bytes: usize,
    max_vertices: usize,
    max_triangles: usize,
    max_objects: usize,
    max_components: usize,
    max_model_metadata: usize,
    max_compression_ratio: u64,
}

impl ThreeMfLimits {
    /// Constructs strictly positive archive, XML, mesh, object, component, and metadata limits.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        max_input_bytes: usize,
        max_entries: usize,
        max_total_uncompressed_bytes: u64,
        max_model_xml_bytes: usize,
        max_vertices: usize,
        max_triangles: usize,
        max_objects: usize,
        max_components: usize,
        max_model_metadata: usize,
        max_compression_ratio: u64,
    ) -> Result<Self, ThreeMfError> {
        if max_input_bytes == 0
            || max_entries == 0
            || max_total_uncompressed_bytes == 0
            || max_model_xml_bytes == 0
            || max_vertices == 0
            || max_triangles == 0
            || max_objects == 0
            || max_components == 0
            || max_model_metadata == 0
            || max_compression_ratio == 0
        {
            return Err(ThreeMfError::InvalidLimits);
        }
        Ok(Self {
            max_input_bytes,
            max_entries,
            max_total_uncompressed_bytes,
            max_model_xml_bytes,
            max_vertices,
            max_triangles,
            max_objects,
            max_components,
            max_model_metadata,
            max_compression_ratio,
        })
    }
}

/// Provisional canonical-millimetre mesh evidence emitted by the 3MF spike.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ThreeMfMeshEvidence {
    algorithm_version: &'static str,
    detected_format: ModelFormat,
    representation: RepresentationBasis,
    source_units: ModelLengthUnit,
    unit_resolution: UnitResolutionMethod,
    unit_was_explicit: bool,
    model_metadata_count: usize,
    preserved_model_metadata_count: usize,
    mesh_object_count: usize,
    mesh_object_id: u32,
    component_object_count: usize,
    component_chain: Vec<ThreeMfComponentEvidence>,
    build_item_count: usize,
    build_object_id: u32,
    build_transform_source_units: [f64; 12],
    build_transform_applied: bool,
    triangle_count: usize,
    manifold: bool,
    watertight: bool,
    consistently_wound: bool,
    aabb_extents_mm: MeshVector3,
    surface_area_mm2: f64,
    enclosed_volume_mm3: Option<f64>,
    center_of_mass_mm: Option<MeshVector3>,
    warnings: Vec<GeometryWarningCode>,
}

/// One retained link in the leaf-to-build linear 3MF component chain.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ThreeMfComponentEvidence {
    object_id: u32,
    referenced_object_id: u32,
    transform_source_units: [f64; 12],
    transform_applied: bool,
}

impl ThreeMfComponentEvidence {
    /// Returns the component object's package-local ID.
    #[must_use]
    pub const fn object_id(&self) -> u32 {
        self.object_id
    }

    /// Returns the immediately preceding object ID referenced by this component.
    #[must_use]
    pub const fn referenced_object_id(&self) -> u32 {
        self.referenced_object_id
    }

    /// Returns the exact 3MF row-vector transform in source units.
    #[must_use]
    pub const fn transform_source_units(&self) -> [f64; 12] {
        self.transform_source_units
    }

    /// Returns whether this link applies a non-identity transform.
    #[must_use]
    pub const fn transform_applied(&self) -> bool {
        self.transform_applied
    }
}

impl ThreeMfMeshEvidence {
    /// Returns the parser algorithm identity.
    #[must_use]
    pub const fn algorithm_version(&self) -> &str {
        self.algorithm_version
    }

    /// Returns the content-detected package format.
    #[must_use]
    pub const fn detected_format(&self) -> ModelFormat {
        self.detected_format
    }

    /// Returns the non-B-rep representation basis.
    #[must_use]
    pub const fn representation(&self) -> RepresentationBasis {
        self.representation
    }

    /// Returns the model unit declared by, or defaulted under, the 3MF core model.
    #[must_use]
    pub const fn source_units(&self) -> ModelLengthUnit {
        self.source_units
    }

    /// Returns how the source units were resolved.
    #[must_use]
    pub const fn unit_resolution(&self) -> UnitResolutionMethod {
        self.unit_resolution
    }

    /// Returns whether the model supplied an explicit `unit` attribute.
    #[must_use]
    pub const fn unit_was_explicit(&self) -> bool {
        self.unit_was_explicit
    }

    /// Returns the count of bounded model-level metadata entries observed without retaining text.
    #[must_use]
    pub const fn model_metadata_count(&self) -> usize {
        self.model_metadata_count
    }

    /// Returns how many model-level metadata entries requested preservation.
    #[must_use]
    pub const fn preserved_model_metadata_count(&self) -> usize {
        self.preserved_model_metadata_count
    }

    /// Returns the leaf mesh-object count retained by this slice.
    #[must_use]
    pub const fn mesh_object_count(&self) -> usize {
        self.mesh_object_count
    }

    /// Returns the leaf mesh object's package-local ID.
    #[must_use]
    pub const fn mesh_object_id(&self) -> u32 {
        self.mesh_object_id
    }

    /// Returns the component-object count retained by this bounded slice.
    #[must_use]
    pub const fn component_object_count(&self) -> usize {
        self.component_object_count
    }

    /// Returns every retained component link in leaf-to-build application order.
    #[must_use]
    pub fn component_chain(&self) -> &[ThreeMfComponentEvidence] {
        &self.component_chain
    }

    /// Returns the build-item count retained by this slice.
    #[must_use]
    pub const fn build_item_count(&self) -> usize {
        self.build_item_count
    }

    /// Returns the object ID referenced by the retained build item.
    #[must_use]
    pub const fn build_object_id(&self) -> u32 {
        self.build_object_id
    }

    /// Returns the retained 3MF row-vector transform in source units.
    #[must_use]
    pub const fn build_transform_source_units(&self) -> [f64; 12] {
        self.build_transform_source_units
    }

    /// Returns whether a non-identity build transform was applied.
    #[must_use]
    pub const fn build_transform_applied(&self) -> bool {
        self.build_transform_applied
    }

    /// Returns the analyzed triangle count.
    #[must_use]
    pub const fn triangle_count(&self) -> usize {
        self.triangle_count
    }

    /// Returns whether no undirected edge is used by more than two triangles.
    #[must_use]
    pub const fn manifold(&self) -> bool {
        self.manifold
    }

    /// Returns whether every undirected edge is used by exactly two triangles.
    #[must_use]
    pub const fn watertight(&self) -> bool {
        self.watertight
    }

    /// Returns whether every paired edge is traversed in opposite directions.
    #[must_use]
    pub const fn consistently_wound(&self) -> bool {
        self.consistently_wound
    }

    /// Returns axis-aligned extents in canonical millimetres.
    #[must_use]
    pub const fn aabb_extents_mm(&self) -> MeshVector3 {
        self.aabb_extents_mm
    }

    /// Returns triangle area in square millimetres.
    #[must_use]
    pub const fn surface_area_mm2(&self) -> f64 {
        self.surface_area_mm2
    }

    /// Returns enclosed volume only for a closed, consistently wound mesh.
    #[must_use]
    pub const fn enclosed_volume_mm3(&self) -> Option<f64> {
        self.enclosed_volume_mm3
    }

    /// Returns the closed-mesh centroid in canonical millimetres.
    #[must_use]
    pub const fn center_of_mass_mm(&self) -> Option<MeshVector3> {
        self.center_of_mass_mm
    }

    /// Returns deterministic, sanitized review warnings.
    #[must_use]
    pub fn warnings(&self) -> &[GeometryWarningCode] {
        &self.warnings
    }
}

/// Sanitized failure from bounded 3MF package analysis.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ThreeMfError {
    /// A parser limit was zero.
    InvalidLimits,
    /// The original package exceeded its explicit byte limit.
    InputLimitExceeded,
    /// ZIP framing or required OPC parts were invalid.
    InvalidPackage,
    /// The archive exceeded entry, expansion, or compression-ratio limits.
    ArchiveLimitExceeded,
    /// An entry was encrypted, linked, unsafe, duplicated, or externally targeted.
    UnsafePackage,
    /// An entry used a compression method outside Store/Deflate.
    UnsupportedCompression,
    /// Required XML was malformed or exceeded its explicit byte limit.
    InvalidXml,
    /// The model requires an extension this slice does not implement.
    UnsupportedRequiredExtension,
    /// The package uses a 3MF structure outside this slice's bounded object contract.
    UnsupportedModelStructure,
    /// The model unit was not a supported 3MF core unit.
    UnsupportedUnit,
    /// A numeric token was invalid or non-finite.
    InvalidNumber,
    /// A transform was singular or reflected geometry outside this slice's policy.
    UnsupportedTransform,
    /// A vertex, triangle, object, component, or metadata count exceeded its explicit limit.
    EntityLimitExceeded,
    /// A triangle had effectively zero area.
    DegenerateTriangle,
    /// The selected build mesh contained no triangles.
    EmptyMesh,
}

impl ThreeMfError {
    /// Returns a stable, content-free diagnostic code.
    #[must_use]
    pub const fn diagnostic_code(self) -> &'static str {
        match self {
            Self::InvalidLimits => "THREE_MF_INVALID_LIMITS",
            Self::InputLimitExceeded => "THREE_MF_INPUT_LIMIT_EXCEEDED",
            Self::InvalidPackage => "THREE_MF_INVALID_PACKAGE",
            Self::ArchiveLimitExceeded => "THREE_MF_ARCHIVE_LIMIT_EXCEEDED",
            Self::UnsafePackage => "THREE_MF_UNSAFE_PACKAGE",
            Self::UnsupportedCompression => "THREE_MF_UNSUPPORTED_COMPRESSION",
            Self::InvalidXml => "THREE_MF_INVALID_XML",
            Self::UnsupportedRequiredExtension => "THREE_MF_UNSUPPORTED_REQUIRED_EXTENSION",
            Self::UnsupportedModelStructure => "THREE_MF_UNSUPPORTED_MODEL_STRUCTURE",
            Self::UnsupportedUnit => "THREE_MF_UNSUPPORTED_UNIT",
            Self::InvalidNumber => "THREE_MF_INVALID_NUMBER",
            Self::UnsupportedTransform => "THREE_MF_UNSUPPORTED_TRANSFORM",
            Self::EntityLimitExceeded => "THREE_MF_ENTITY_LIMIT_EXCEEDED",
            Self::DegenerateTriangle => "THREE_MF_DEGENERATE_TRIANGLE",
            Self::EmptyMesh => "THREE_MF_EMPTY_MESH",
        }
    }
}

impl fmt::Display for ThreeMfError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.diagnostic_code())
    }
}

impl Error for ThreeMfError {}

#[derive(Clone, Copy, Debug)]
struct Transform([f64; 12]);

impl Transform {
    const IDENTITY: Self = Self([1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0]);

    fn apply(self, vertex: MeshVector3) -> Result<MeshVector3, ThreeMfError> {
        let matrix = self.0;
        let transformed = MeshVector3::new(
            vertex.x * matrix[0] + vertex.y * matrix[3] + vertex.z * matrix[6] + matrix[9],
            vertex.x * matrix[1] + vertex.y * matrix[4] + vertex.z * matrix[7] + matrix[10],
            vertex.x * matrix[2] + vertex.y * matrix[5] + vertex.z * matrix[8] + matrix[11],
        );
        if transformed.is_finite() {
            Ok(transformed)
        } else {
            Err(ThreeMfError::InvalidNumber)
        }
    }

    fn is_identity(self) -> bool {
        self.0 == Self::IDENTITY.0
    }

    fn has_positive_determinant(self) -> bool {
        let matrix = self.0;
        let determinant = matrix[0] * (matrix[4] * matrix[8] - matrix[5] * matrix[7])
            - matrix[1] * (matrix[3] * matrix[8] - matrix[5] * matrix[6])
            + matrix[2] * (matrix[3] * matrix[7] - matrix[4] * matrix[6]);
        determinant.is_finite() && determinant > 0.0
    }
}

#[derive(Debug)]
struct ParsedModel {
    source_units: ModelLengthUnit,
    unit_was_explicit: bool,
    model_metadata_count: usize,
    preserved_model_metadata_count: usize,
    mesh_object_id: u32,
    vertices: Vec<MeshVector3>,
    triangle_indices: Vec<[usize; 3]>,
    component_chain: Vec<ParsedComponentLink>,
    build_object_id: u32,
    build_transform: Transform,
}

#[derive(Debug)]
struct ParsedComponentLink {
    object_id: u32,
    referenced_object_id: u32,
    transform: Transform,
}

/// Validates and measures one in-memory 3MF package without extracting it to disk.
pub fn analyze_3mf(
    bytes: &[u8],
    limits: ThreeMfLimits,
) -> Result<ThreeMfMeshEvidence, ThreeMfError> {
    if bytes.len() > limits.max_input_bytes {
        return Err(ThreeMfError::InputLimitExceeded);
    }

    let mut archive =
        ZipArchive::new(Cursor::new(bytes)).map_err(|_| ThreeMfError::InvalidPackage)?;
    preflight_archive(&mut archive, limits)?;

    let relationships = read_part(
        &mut archive,
        ROOT_RELATIONSHIPS_PART,
        limits.max_model_xml_bytes,
    )?;
    let model_part = parse_root_relationships(&relationships)?;
    let content_types = read_part(&mut archive, CONTENT_TYPES_PART, limits.max_model_xml_bytes)?;
    require_model_content_type(&content_types, &model_part)?;
    let model_xml = read_part(&mut archive, &model_part, limits.max_model_xml_bytes)?;
    let parsed = parse_model(&model_xml, limits)?;
    let expected_build_object_id = parsed
        .component_chain
        .last()
        .map_or(parsed.mesh_object_id, |component| component.object_id);
    if expected_build_object_id != parsed.build_object_id {
        return Err(ThreeMfError::UnsupportedModelStructure);
    }

    let millimetres_per_unit = millimetres_per_unit(parsed.source_units)?;
    let mut triangles = Vec::with_capacity(parsed.triangle_indices.len());
    for indices in parsed.triangle_indices {
        let mut vertices = [MeshVector3::new(0.0, 0.0, 0.0); 3];
        for (target, source_index) in vertices.iter_mut().zip(indices) {
            let source = parsed
                .vertices
                .get(source_index)
                .copied()
                .ok_or(ThreeMfError::UnsupportedModelStructure)?;
            let mut transformed = source;
            for component in &parsed.component_chain {
                transformed = component.transform.apply(transformed)?;
            }
            *target = parsed
                .build_transform
                .apply(transformed)?
                .scale(millimetres_per_unit);
        }
        let triangle = Triangle(vertices);
        validate_triangle(triangle).map_err(map_mesh_error)?;
        triangles.push(triangle);
    }
    if triangles.is_empty() {
        return Err(ThreeMfError::EmptyMesh);
    }

    let analysis = analyze_triangles(&triangles).map_err(map_mesh_error)?;
    let mut warnings = mesh_warning_codes(&analysis);
    if parsed.model_metadata_count > 0 {
        warnings.push(warning("THREE_MF_METADATA_NOT_INTERPRETED"));
    }
    Ok(ThreeMfMeshEvidence {
        algorithm_version: THREE_MF_ANALYZER_VERSION,
        detected_format: ModelFormat::ThreeMf,
        representation: RepresentationBasis::Mesh,
        source_units: parsed.source_units,
        unit_resolution: UnitResolutionMethod::Declared,
        unit_was_explicit: parsed.unit_was_explicit,
        model_metadata_count: parsed.model_metadata_count,
        preserved_model_metadata_count: parsed.preserved_model_metadata_count,
        mesh_object_count: 1,
        mesh_object_id: parsed.mesh_object_id,
        component_object_count: parsed.component_chain.len(),
        component_chain: parsed
            .component_chain
            .into_iter()
            .map(|component| ThreeMfComponentEvidence {
                object_id: component.object_id,
                referenced_object_id: component.referenced_object_id,
                transform_source_units: component.transform.0,
                transform_applied: !component.transform.is_identity(),
            })
            .collect(),
        build_item_count: 1,
        build_object_id: parsed.build_object_id,
        build_transform_source_units: parsed.build_transform.0,
        build_transform_applied: !parsed.build_transform.is_identity(),
        triangle_count: triangles.len(),
        manifold: analysis.manifold,
        watertight: analysis.watertight,
        consistently_wound: analysis.consistently_wound,
        aabb_extents_mm: analysis.aabb_extents,
        surface_area_mm2: analysis.surface_area,
        enclosed_volume_mm3: analysis.enclosed_volume,
        center_of_mass_mm: analysis.center_of_mass,
        warnings,
    })
}

fn preflight_archive(
    archive: &mut ZipArchive<Cursor<&[u8]>>,
    limits: ThreeMfLimits,
) -> Result<(), ThreeMfError> {
    if archive.is_empty() || archive.len() > limits.max_entries {
        return Err(ThreeMfError::ArchiveLimitExceeded);
    }
    let mut names = BTreeSet::new();
    let mut folded_names = BTreeSet::new();
    let mut total_uncompressed = 0_u64;
    for index in 0..archive.len() {
        let entry = archive
            .by_index_raw(index)
            .map_err(|_| ThreeMfError::InvalidPackage)?;
        let name = entry.name();
        if entry.encrypted()
            || !entry.is_file()
            || entry.unix_mode().is_some_and(|mode| {
                let file_type = mode & 0o170_000;
                file_type != 0 && file_type != 0o100_000
            })
            || entry.enclosed_name().is_none()
            || name.starts_with('/')
            || name.contains('\\')
            || name
                .split('/')
                .any(|segment| segment.is_empty() || segment == "." || segment == "..")
        {
            return Err(ThreeMfError::UnsafePackage);
        }
        if !names.insert(name.to_owned()) || !folded_names.insert(name.to_ascii_lowercase()) {
            return Err(ThreeMfError::UnsafePackage);
        }
        if !matches!(
            entry.compression(),
            CompressionMethod::Stored | CompressionMethod::Deflated
        ) {
            return Err(ThreeMfError::UnsupportedCompression);
        }
        let size = entry.size();
        let compressed_size = entry.compressed_size();
        total_uncompressed = total_uncompressed
            .checked_add(size)
            .ok_or(ThreeMfError::ArchiveLimitExceeded)?;
        if total_uncompressed > limits.max_total_uncompressed_bytes
            || (size > 0
                && (compressed_size == 0
                    || size
                        > compressed_size
                            .checked_mul(limits.max_compression_ratio)
                            .ok_or(ThreeMfError::ArchiveLimitExceeded)?))
        {
            return Err(ThreeMfError::ArchiveLimitExceeded);
        }
    }
    if !names.contains(CONTENT_TYPES_PART) || !names.contains(ROOT_RELATIONSHIPS_PART) {
        return Err(ThreeMfError::InvalidPackage);
    }
    Ok(())
}

fn read_part(
    archive: &mut ZipArchive<Cursor<&[u8]>>,
    name: &str,
    limit: usize,
) -> Result<Vec<u8>, ThreeMfError> {
    let mut entry = archive
        .by_name(name)
        .map_err(|_| ThreeMfError::InvalidPackage)?;
    let declared_size = usize::try_from(entry.size()).map_err(|_| ThreeMfError::InvalidXml)?;
    if declared_size > limit {
        return Err(ThreeMfError::InvalidXml);
    }
    let mut bytes = Vec::with_capacity(declared_size);
    entry
        .by_ref()
        .take(u64::try_from(limit).unwrap_or(u64::MAX).saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|_| ThreeMfError::InvalidPackage)?;
    if bytes.len() != declared_size || bytes.len() > limit {
        return Err(ThreeMfError::InvalidXml);
    }
    Ok(bytes)
}

fn parse_root_relationships(xml: &[u8]) -> Result<String, ThreeMfError> {
    let mut reader = xml_reader(xml);
    let mut buffer = Vec::new();
    let mut root_seen = false;
    let mut root_closed = false;
    let mut depth = 0_u8;
    let mut model_target = None;
    loop {
        match reader
            .read_event_into(&mut buffer)
            .map_err(|_| ThreeMfError::InvalidXml)?
        {
            Event::Start(element) if element.name().as_ref() == b"Relationships" => {
                if root_seen || root_closed || depth != 0 {
                    return Err(ThreeMfError::InvalidPackage);
                }
                require_only_package_attributes(&element, &[b"xmlns"])?;
                if required_attribute(&element, b"xmlns")? != RELATIONSHIPS_NAMESPACE {
                    return Err(ThreeMfError::InvalidPackage);
                }
                root_seen = true;
                depth = 1;
            }
            Event::Start(element) if element.name().as_ref() == b"Relationship" => {
                if !root_seen || root_closed || depth != 1 {
                    return Err(ThreeMfError::InvalidPackage);
                }
                require_only_package_attributes(
                    &element,
                    &[b"Id", b"Type", b"Target", b"TargetMode"],
                )?;
                let _id = required_attribute(&element, b"Id")?;
                let relationship_type = required_attribute(&element, b"Type")?;
                let target = required_attribute(&element, b"Target")?;
                let target_mode = optional_attribute(&element, b"TargetMode")?;
                if target_mode
                    .as_deref()
                    .is_some_and(|mode| mode != "Internal")
                {
                    return Err(ThreeMfError::UnsafePackage);
                }
                if relationship_type == RELATIONSHIP_TYPE {
                    if model_target.is_some() {
                        return Err(ThreeMfError::InvalidPackage);
                    }
                    model_target = Some(normalize_part_target(&target)?);
                }
                depth = 2;
            }
            Event::End(element) if element.name().as_ref() == b"Relationship" && depth == 2 => {
                depth = 1;
            }
            Event::End(element) if element.name().as_ref() == b"Relationships" => {
                if !root_seen || root_closed || depth != 1 {
                    return Err(ThreeMfError::InvalidPackage);
                }
                root_closed = true;
                depth = 0;
            }
            Event::Text(text) if text.iter().all(u8::is_ascii_whitespace) => {}
            Event::Decl(_) | Event::Comment(_) | Event::PI(_) => {}
            Event::Eof => break,
            Event::DocType(_) | Event::GeneralRef(_) => return Err(ThreeMfError::InvalidXml),
            _ => return Err(ThreeMfError::InvalidPackage),
        }
        buffer.clear();
    }
    if !root_closed || depth != 0 {
        return Err(ThreeMfError::InvalidPackage);
    }
    model_target.ok_or(ThreeMfError::InvalidPackage)
}

fn require_model_content_type(xml: &[u8], model_part: &str) -> Result<(), ThreeMfError> {
    let mut reader = xml_reader(xml);
    let mut buffer = Vec::new();
    let expected_part_name = format!("/{model_part}");
    let expected_extension = model_part.rsplit_once('.').map(|(_, extension)| extension);
    let mut root_seen = false;
    let mut root_closed = false;
    let mut depth = 0_u8;
    let mut override_type = None;
    let mut default_type = None;
    loop {
        match reader
            .read_event_into(&mut buffer)
            .map_err(|_| ThreeMfError::InvalidXml)?
        {
            Event::Start(element) if element.name().as_ref() == b"Types" => {
                if root_seen || root_closed || depth != 0 {
                    return Err(ThreeMfError::InvalidPackage);
                }
                require_only_package_attributes(&element, &[b"xmlns"])?;
                if required_attribute(&element, b"xmlns")? != CONTENT_TYPES_NAMESPACE {
                    return Err(ThreeMfError::InvalidPackage);
                }
                root_seen = true;
                depth = 1;
            }
            Event::Start(element) if element.name().as_ref() == b"Override" => {
                if !root_seen || root_closed || depth != 1 {
                    return Err(ThreeMfError::InvalidPackage);
                }
                require_only_package_attributes(&element, &[b"PartName", b"ContentType"])?;
                if required_attribute(&element, b"PartName")? == expected_part_name {
                    if override_type.is_some() {
                        return Err(ThreeMfError::InvalidPackage);
                    }
                    override_type = Some(required_attribute(&element, b"ContentType")?);
                }
                depth = 2;
            }
            Event::Start(element) if element.name().as_ref() == b"Default" => {
                if !root_seen || root_closed || depth != 1 {
                    return Err(ThreeMfError::InvalidPackage);
                }
                require_only_package_attributes(&element, &[b"Extension", b"ContentType"])?;
                if Some(required_attribute(&element, b"Extension")?.as_str()) == expected_extension
                {
                    if default_type.is_some() {
                        return Err(ThreeMfError::InvalidPackage);
                    }
                    default_type = Some(required_attribute(&element, b"ContentType")?);
                }
                depth = 2;
            }
            Event::End(element)
                if matches!(element.name().as_ref(), b"Override" | b"Default") && depth == 2 =>
            {
                depth = 1;
            }
            Event::End(element) if element.name().as_ref() == b"Types" => {
                if depth != 1 {
                    return Err(ThreeMfError::InvalidPackage);
                }
                root_closed = true;
                depth = 0;
            }
            Event::Text(text) if text.iter().all(u8::is_ascii_whitespace) => {}
            Event::Decl(_) | Event::Comment(_) | Event::PI(_) => {}
            Event::Eof => break,
            Event::DocType(_) | Event::GeneralRef(_) => return Err(ThreeMfError::InvalidXml),
            _ => return Err(ThreeMfError::InvalidPackage),
        }
        buffer.clear();
    }
    if root_closed
        && depth == 0
        && override_type
            .or(default_type)
            .is_some_and(|content_type| content_type == MODEL_CONTENT_TYPE)
    {
        Ok(())
    } else {
        Err(ThreeMfError::InvalidPackage)
    }
}

fn parse_model(xml: &[u8], limits: ThreeMfLimits) -> Result<ParsedModel, ThreeMfError> {
    let mut reader = xml_reader(xml);
    let mut buffer = Vec::new();
    let mut stack = Vec::<Vec<u8>>::new();
    let mut source_units = ModelLengthUnit::Millimeter;
    let mut unit_was_explicit = false;
    let mut model_metadata_names = BTreeSet::new();
    let mut model_metadata_count = 0_usize;
    let mut preserved_model_metadata_count = 0_usize;
    let mut object_count = 0_usize;
    let mut object_ids = BTreeSet::new();
    let mut current_object_id = None;
    let mut last_object_id = None;
    let mut expected_component_reference = None;
    let mut mesh_object_id = None;
    let mut vertices = Vec::new();
    let mut triangle_indices = Vec::new();
    let mut component_chain = Vec::new();
    let mut component_count = 0_usize;
    let mut build_object_id = None;
    let mut build_transform = Transform::IDENTITY;
    let mut resources_seen = false;
    let mut resources_closed = false;
    let mut mesh_seen = false;
    let mut vertices_closed = false;
    let mut triangles_closed = false;
    let mut components_seen = false;
    let mut components_closed = false;
    let mut build_seen = false;
    let mut model_closed = false;

    loop {
        match reader
            .read_event_into(&mut buffer)
            .map_err(|_| ThreeMfError::InvalidXml)?
        {
            Event::Start(element) => {
                let name = element.name().as_ref().to_vec();
                let parent = stack.last().map(Vec::as_slice);
                match (name.as_slice(), parent) {
                    (b"model", None) if !model_closed => {
                        if optional_attribute(&element, b"requiredextensions")?
                            .is_some_and(|value| !value.trim().is_empty())
                        {
                            return Err(ThreeMfError::UnsupportedRequiredExtension);
                        }
                        require_only_model_attributes(
                            &element,
                            &[b"xmlns", b"unit", b"xml:lang", b"requiredextensions"],
                        )?;
                        if required_attribute(&element, b"xmlns")? != CORE_NAMESPACE {
                            return Err(ThreeMfError::UnsupportedModelStructure);
                        }
                        if let Some(unit) = optional_attribute(&element, b"unit")? {
                            source_units = parse_unit(&unit)?;
                            unit_was_explicit = true;
                        }
                    }
                    (b"resources", Some(b"model")) if !resources_closed => {
                        require_only_model_attributes(&element, &[])?;
                        resources_seen = true;
                    }
                    (b"metadata", Some(b"model")) if !resources_seen => {
                        if model_metadata_count == limits.max_model_metadata {
                            return Err(ThreeMfError::EntityLimitExceeded);
                        }
                        require_only_model_attributes(&element, &[b"name", b"preserve"])?;
                        let name = required_attribute(&element, b"name")?;
                        if !is_well_known_model_metadata_name(&name)
                            || !model_metadata_names.insert(name)
                        {
                            return Err(ThreeMfError::UnsupportedModelStructure);
                        }
                        let preserve = optional_attribute(&element, b"preserve")?
                            .map_or(Ok(false), |value| parse_xml_boolean(&value))?;
                        model_metadata_count += 1;
                        preserved_model_metadata_count += usize::from(preserve);
                    }
                    (b"object", Some(b"resources")) if current_object_id.is_none() => {
                        if object_count == limits.max_objects {
                            return Err(ThreeMfError::EntityLimitExceeded);
                        }
                        if object_count == 1
                            && (!mesh_seen
                                || !vertices_closed
                                || !triangles_closed
                                || vertices.is_empty()
                                || triangle_indices.is_empty())
                        {
                            return Err(ThreeMfError::UnsupportedModelStructure);
                        }
                        require_only_model_attributes(&element, &[b"id", b"type"])?;
                        if optional_attribute(&element, b"type")?
                            .is_some_and(|value| value != "model")
                        {
                            return Err(ThreeMfError::UnsupportedModelStructure);
                        }
                        let id = parse_object_id_attribute(&element, b"id")?;
                        if !object_ids.insert(id) {
                            return Err(ThreeMfError::UnsupportedModelStructure);
                        }
                        if object_count == 0 {
                            mesh_object_id = Some(id);
                        }
                        expected_component_reference = last_object_id;
                        component_count = 0;
                        components_seen = false;
                        components_closed = false;
                        current_object_id = Some(id);
                        object_count += 1;
                    }
                    (b"mesh", Some(b"object"))
                        if current_object_id == mesh_object_id && !mesh_seen =>
                    {
                        require_only_model_attributes(&element, &[])?;
                        mesh_seen = true;
                    }
                    (b"vertices", Some(b"mesh")) if !vertices_closed => {
                        require_only_model_attributes(&element, &[])?;
                    }
                    (b"vertex", Some(b"vertices")) => {
                        require_only_model_attributes(&element, &[b"x", b"y", b"z"])?;
                        if vertices.len() == limits.max_vertices {
                            return Err(ThreeMfError::EntityLimitExceeded);
                        }
                        vertices.push(MeshVector3::new(
                            parse_f64_attribute(&element, b"x")?,
                            parse_f64_attribute(&element, b"y")?,
                            parse_f64_attribute(&element, b"z")?,
                        ));
                    }
                    (b"triangles", Some(b"mesh")) if vertices_closed && !triangles_closed => {
                        require_only_model_attributes(&element, &[])?;
                    }
                    (b"triangle", Some(b"triangles")) => {
                        require_only_model_attributes(&element, &[b"v1", b"v2", b"v3"])?;
                        if triangle_indices.len() == limits.max_triangles {
                            return Err(ThreeMfError::EntityLimitExceeded);
                        }
                        triangle_indices.push([
                            parse_usize_attribute(&element, b"v1")?,
                            parse_usize_attribute(&element, b"v2")?,
                            parse_usize_attribute(&element, b"v3")?,
                        ]);
                    }
                    (b"components", Some(b"object"))
                        if current_object_id != mesh_object_id && !components_seen =>
                    {
                        require_only_model_attributes(&element, &[])?;
                        components_seen = true;
                    }
                    (b"component", Some(b"components")) => {
                        if component_count != 0 {
                            return Err(ThreeMfError::UnsupportedModelStructure);
                        }
                        if component_chain.len() == limits.max_components {
                            return Err(ThreeMfError::EntityLimitExceeded);
                        }
                        require_only_model_attributes(&element, &[b"objectid", b"transform"])?;
                        let referenced_id = parse_object_id_attribute(&element, b"objectid")?;
                        if Some(referenced_id) != expected_component_reference {
                            return Err(ThreeMfError::UnsupportedModelStructure);
                        }
                        component_chain.push(ParsedComponentLink {
                            object_id: current_object_id
                                .ok_or(ThreeMfError::UnsupportedModelStructure)?,
                            referenced_object_id: referenced_id,
                            transform: optional_attribute(&element, b"transform")?
                                .map_or(Ok(Transform::IDENTITY), |value| parse_transform(&value))?,
                        });
                        component_count += 1;
                    }
                    (b"build", Some(b"model")) if resources_closed && !build_seen => {
                        require_only_model_attributes(&element, &[])?;
                        build_seen = true;
                    }
                    (b"item", Some(b"build")) if build_object_id.is_none() => {
                        require_only_model_attributes(&element, &[b"objectid", b"transform"])?;
                        build_object_id = Some(parse_object_id_attribute(&element, b"objectid")?);
                        if let Some(transform) = optional_attribute(&element, b"transform")? {
                            build_transform = parse_transform(&transform)?;
                        }
                    }
                    _ => return Err(ThreeMfError::UnsupportedModelStructure),
                }
                stack.push(name);
            }
            Event::End(element) => {
                let expected = stack.pop().ok_or(ThreeMfError::InvalidXml)?;
                if expected.as_slice() != element.name().as_ref() {
                    return Err(ThreeMfError::InvalidXml);
                }
                match expected.as_slice() {
                    b"vertices" => vertices_closed = true,
                    b"triangles" => triangles_closed = true,
                    b"components" => components_closed = true,
                    b"object" => {
                        let object_id =
                            current_object_id.ok_or(ThreeMfError::UnsupportedModelStructure)?;
                        if Some(object_id) == mesh_object_id {
                            if !mesh_seen
                                || !vertices_closed
                                || !triangles_closed
                                || vertices.is_empty()
                                || triangle_indices.is_empty()
                                || components_seen
                                || components_closed
                                || component_count != 0
                            {
                                return Err(ThreeMfError::UnsupportedModelStructure);
                            }
                        } else if !components_seen || !components_closed || component_count != 1 {
                            return Err(ThreeMfError::UnsupportedModelStructure);
                        }
                        last_object_id = Some(object_id);
                        current_object_id = None;
                        expected_component_reference = None;
                    }
                    b"resources" => resources_closed = true,
                    b"model" => model_closed = true,
                    _ => {}
                }
            }
            Event::Text(text) if stack.last().is_some_and(|name| name == b"metadata") => {
                text.decode().map_err(|_| ThreeMfError::InvalidXml)?;
            }
            Event::CData(text) if stack.last().is_some_and(|name| name == b"metadata") => {
                text.decode().map_err(|_| ThreeMfError::InvalidXml)?;
            }
            Event::Text(text) if text.iter().all(u8::is_ascii_whitespace) => {}
            Event::Decl(_) | Event::Comment(_) | Event::PI(_) => {}
            Event::Eof => break,
            Event::DocType(_) | Event::GeneralRef(_) => return Err(ThreeMfError::InvalidXml),
            _ => return Err(ThreeMfError::UnsupportedModelStructure),
        }
        buffer.clear();
    }

    if !stack.is_empty()
        || !model_closed
        || !resources_seen
        || !resources_closed
        || !mesh_seen
        || !vertices_closed
        || !triangles_closed
        || !build_seen
        || mesh_object_id.is_none()
        || build_object_id.is_none()
        || vertices.is_empty()
        || triangle_indices.is_empty()
        || object_count == 0
        || component_chain.len() != object_count - 1
        || build_object_id != last_object_id
    {
        return Err(ThreeMfError::UnsupportedModelStructure);
    }
    Ok(ParsedModel {
        source_units,
        unit_was_explicit,
        model_metadata_count,
        preserved_model_metadata_count,
        mesh_object_id: mesh_object_id.expect("checked above"),
        vertices,
        triangle_indices,
        component_chain,
        build_object_id: build_object_id.expect("checked above"),
        build_transform,
    })
}

fn is_well_known_model_metadata_name(name: &str) -> bool {
    matches!(
        name,
        "Title"
            | "Designer"
            | "Description"
            | "Copyright"
            | "LicenseTerms"
            | "Rating"
            | "CreationDate"
            | "ModificationDate"
            | "Application"
    )
}

fn parse_xml_boolean(value: &str) -> Result<bool, ThreeMfError> {
    match value {
        "0" | "false" => Ok(false),
        "1" | "true" => Ok(true),
        _ => Err(ThreeMfError::UnsupportedModelStructure),
    }
}

fn xml_reader(xml: &[u8]) -> Reader<&[u8]> {
    let mut reader = Reader::from_reader(xml);
    let config = reader.config_mut();
    config.trim_text(true);
    config.expand_empty_elements = true;
    config.check_end_names = true;
    reader
}

fn required_attribute(element: &BytesStart<'_>, name: &[u8]) -> Result<String, ThreeMfError> {
    optional_attribute(element, name)?.ok_or(ThreeMfError::InvalidXml)
}

fn optional_attribute(
    element: &BytesStart<'_>,
    name: &[u8],
) -> Result<Option<String>, ThreeMfError> {
    let mut found = None;
    for attribute in element.attributes().with_checks(true) {
        let attribute = attribute.map_err(|_| ThreeMfError::InvalidXml)?;
        if attribute.key.as_ref() == name {
            if found.is_some() {
                return Err(ThreeMfError::InvalidXml);
            }
            found = Some(
                attribute
                    .decoded_and_normalized_value(XmlVersion::Implicit1_0, element.decoder())
                    .map_err(|_| ThreeMfError::InvalidXml)?
                    .into_owned(),
            );
        }
    }
    Ok(found)
}

fn require_only_package_attributes(
    element: &BytesStart<'_>,
    allowed: &[&[u8]],
) -> Result<(), ThreeMfError> {
    require_only_attributes(element, allowed, ThreeMfError::InvalidPackage)
}

fn require_only_model_attributes(
    element: &BytesStart<'_>,
    allowed: &[&[u8]],
) -> Result<(), ThreeMfError> {
    require_only_attributes(element, allowed, ThreeMfError::UnsupportedModelStructure)
}

fn require_only_attributes(
    element: &BytesStart<'_>,
    allowed: &[&[u8]],
    error: ThreeMfError,
) -> Result<(), ThreeMfError> {
    let mut seen = BTreeSet::new();
    for attribute in element.attributes().with_checks(true) {
        let attribute = attribute.map_err(|_| ThreeMfError::InvalidXml)?;
        let name = attribute.key.as_ref();
        if !allowed.contains(&name) || !seen.insert(name.to_vec()) {
            return Err(error);
        }
    }
    Ok(())
}

fn normalize_part_target(target: &str) -> Result<String, ThreeMfError> {
    let relative = target
        .strip_prefix('/')
        .ok_or(ThreeMfError::UnsafePackage)?;
    if relative.is_empty()
        || relative.contains('\\')
        || relative
            .split('/')
            .any(|segment| segment.is_empty() || segment == "." || segment == "..")
    {
        return Err(ThreeMfError::UnsafePackage);
    }
    Ok(relative.to_owned())
}

fn parse_unit(unit: &str) -> Result<ModelLengthUnit, ThreeMfError> {
    match unit {
        "micron" => Ok(ModelLengthUnit::Micrometer),
        "millimeter" => Ok(ModelLengthUnit::Millimeter),
        "centimeter" => Ok(ModelLengthUnit::Centimeter),
        "meter" => Ok(ModelLengthUnit::Meter),
        "inch" => Ok(ModelLengthUnit::Inch),
        "foot" => Ok(ModelLengthUnit::Foot),
        _ => Err(ThreeMfError::UnsupportedUnit),
    }
}

const fn millimetres_per_unit(unit: ModelLengthUnit) -> Result<f64, ThreeMfError> {
    match unit {
        ModelLengthUnit::Micrometer => Ok(0.001),
        ModelLengthUnit::Millimeter => Ok(1.0),
        ModelLengthUnit::Centimeter => Ok(10.0),
        ModelLengthUnit::Meter => Ok(1_000.0),
        ModelLengthUnit::Inch => Ok(25.4),
        ModelLengthUnit::Foot => Ok(304.8),
        ModelLengthUnit::Unknown => Err(ThreeMfError::UnsupportedUnit),
    }
}

fn parse_object_id_attribute(element: &BytesStart<'_>, name: &[u8]) -> Result<u32, ThreeMfError> {
    let value = required_attribute(element, name)?
        .parse()
        .map_err(|_| ThreeMfError::InvalidNumber)?;
    if value == 0 || value > i32::MAX as u32 {
        Err(ThreeMfError::InvalidNumber)
    } else {
        Ok(value)
    }
}

fn parse_usize_attribute(element: &BytesStart<'_>, name: &[u8]) -> Result<usize, ThreeMfError> {
    required_attribute(element, name)?
        .parse()
        .map_err(|_| ThreeMfError::InvalidNumber)
}

fn parse_f64_attribute(element: &BytesStart<'_>, name: &[u8]) -> Result<f64, ThreeMfError> {
    parse_f64(&required_attribute(element, name)?)
}

fn parse_f64(value: &str) -> Result<f64, ThreeMfError> {
    let parsed = value
        .parse::<f64>()
        .map_err(|_| ThreeMfError::InvalidNumber)?;
    if parsed.is_finite() {
        Ok(parsed)
    } else {
        Err(ThreeMfError::InvalidNumber)
    }
}

fn parse_transform(value: &str) -> Result<Transform, ThreeMfError> {
    let values = value
        .split_ascii_whitespace()
        .map(parse_f64)
        .collect::<Result<Vec<_>, _>>()?;
    let matrix: [f64; 12] = values.try_into().map_err(|_| ThreeMfError::InvalidNumber)?;
    let transform = Transform(matrix);
    if transform.has_positive_determinant() {
        Ok(transform)
    } else {
        Err(ThreeMfError::UnsupportedTransform)
    }
}

const fn map_mesh_error(error: MeshAnalysisError) -> ThreeMfError {
    match error {
        MeshAnalysisError::DegenerateTriangle => ThreeMfError::DegenerateTriangle,
        MeshAnalysisError::InvalidNumber => ThreeMfError::InvalidNumber,
    }
}

fn warning(code: &str) -> GeometryWarningCode {
    GeometryWarningCode::new(code).expect("static 3MF warning code must be valid")
}
