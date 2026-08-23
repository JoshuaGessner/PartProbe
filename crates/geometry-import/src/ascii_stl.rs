//! Bounded, path-free ASCII and binary STL comparison spike.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

use partprobe_geometry_core::{
    GeometryWarningCode, ModelFormat, ModelLengthUnit, RepresentationBasis, UnitResolutionMethod,
};
use serde::Serialize;

/// Versioned algorithm identity for the initial ASCII STL comparison spike.
pub const ASCII_STL_ANALYZER_VERSION: &str = "partprobe-ascii-stl-spike-v1";
/// Versioned algorithm identity for the initial binary STL comparison spike.
pub const BINARY_STL_ANALYZER_VERSION: &str = "partprobe-binary-stl-spike-v1";

const DEGENERATE_AREA_EPSILON: f64 = 1.0e-12;
const ZERO_VOLUME_EPSILON: f64 = 1.0e-12;
const BINARY_HEADER_BYTES: usize = 80;
const BINARY_COUNT_BYTES: usize = 4;
const BINARY_TRIANGLE_BYTES: usize = 50;

/// Explicit parser limits applied before and during STL analysis.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AsciiStlLimits {
    max_input_bytes: usize,
    max_triangles: usize,
}

/// STL byte encoding established from parser framing.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StlEncoding {
    /// Line-oriented ASCII STL grammar.
    Ascii,
    /// Fixed-size little-endian binary STL records.
    Binary,
}

impl AsciiStlLimits {
    /// Constructs strictly positive parser limits.
    pub fn new(max_input_bytes: usize, max_triangles: usize) -> Result<Self, AsciiStlError> {
        if max_input_bytes == 0 || max_triangles == 0 {
            return Err(AsciiStlError::InvalidLimits);
        }
        Ok(Self {
            max_input_bytes,
            max_triangles,
        })
    }
}

/// Encoding-neutral name for the STL parser limits.
pub type StlLimits = AsciiStlLimits;

/// Three-dimensional mesh evidence in unresolved STL source-coordinate units.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct MeshVector3 {
    x: f64,
    y: f64,
    z: f64,
}

impl MeshVector3 {
    const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    /// Returns the source-coordinate components.
    #[must_use]
    pub const fn components(self) -> [f64; 3] {
        [self.x, self.y, self.z]
    }

    fn subtract(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }

    fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }

    fn scale(self, factor: f64) -> Self {
        Self::new(self.x * factor, self.y * factor, self.z * factor)
    }

    fn dot(self, other: Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    fn cross(self, other: Self) -> Self {
        Self::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    fn norm(self) -> f64 {
        self.dot(self).sqrt()
    }

    fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.z.is_finite()
    }
}

/// Provisional mesh evidence emitted by the STL comparison spike.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AsciiStlMeshEvidence {
    algorithm_version: &'static str,
    encoding: StlEncoding,
    detected_format: ModelFormat,
    representation: RepresentationBasis,
    source_units: ModelLengthUnit,
    unit_resolution: UnitResolutionMethod,
    triangle_count: usize,
    manifold: bool,
    watertight: bool,
    consistently_wound: bool,
    aabb_extents_source_units: MeshVector3,
    surface_area_source_units_squared: f64,
    enclosed_volume_source_units_cubed: Option<f64>,
    center_of_mass_source_units: Option<MeshVector3>,
    warnings: Vec<GeometryWarningCode>,
}

impl AsciiStlMeshEvidence {
    /// Returns the parser algorithm identity.
    #[must_use]
    pub const fn algorithm_version(&self) -> &str {
        self.algorithm_version
    }

    /// Returns the parser-confirmed STL encoding.
    #[must_use]
    pub const fn encoding(&self) -> StlEncoding {
        self.encoding
    }

    /// Returns the content-detected format.
    #[must_use]
    pub const fn detected_format(&self) -> ModelFormat {
        self.detected_format
    }

    /// Returns the non-B-rep representation basis.
    #[must_use]
    pub const fn representation(&self) -> RepresentationBasis {
        self.representation
    }

    /// Returns the unresolved STL unit state.
    #[must_use]
    pub const fn source_units(&self) -> ModelLengthUnit {
        self.source_units
    }

    /// Returns how source units were resolved.
    #[must_use]
    pub const fn unit_resolution(&self) -> UnitResolutionMethod {
        self.unit_resolution
    }

    /// Returns the parsed triangle count.
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

    /// Returns axis-aligned extents in unresolved STL source-coordinate units.
    #[must_use]
    pub const fn aabb_extents_source_units(&self) -> MeshVector3 {
        self.aabb_extents_source_units
    }

    /// Returns triangle area in unresolved source-coordinate units squared.
    #[must_use]
    pub const fn surface_area_source_units_squared(&self) -> f64 {
        self.surface_area_source_units_squared
    }

    /// Returns enclosed volume only for a closed, consistently wound mesh.
    #[must_use]
    pub const fn enclosed_volume_source_units_cubed(&self) -> Option<f64> {
        self.enclosed_volume_source_units_cubed
    }

    /// Returns the closed-mesh centroid in unresolved source-coordinate units.
    #[must_use]
    pub const fn center_of_mass_source_units(&self) -> Option<MeshVector3> {
        self.center_of_mass_source_units
    }

    /// Returns deterministic, sanitized review warnings.
    #[must_use]
    pub fn warnings(&self) -> &[GeometryWarningCode] {
        &self.warnings
    }
}

/// Encoding-neutral name for provisional STL mesh evidence.
pub type StlMeshEvidence = AsciiStlMeshEvidence;

/// Sanitized failure from bounded STL analysis.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AsciiStlError {
    /// A parser limit was zero.
    InvalidLimits,
    /// Input exceeded the explicit byte limit.
    InputLimitExceeded,
    /// Input was not valid UTF-8 ASCII-style text.
    InvalidText,
    /// Required STL structure was missing or out of order.
    InvalidStructure,
    /// A numeric token was invalid or non-finite.
    InvalidNumber,
    /// Binary per-facet attribute payloads are outside this spike's contract.
    UnsupportedAttributeData,
    /// Triangle count exceeded the explicit limit.
    TriangleLimitExceeded,
    /// A triangle had effectively zero area.
    DegenerateTriangle,
    /// The STL contained no triangles.
    EmptyMesh,
}

impl AsciiStlError {
    /// Returns a stable, content-free diagnostic code.
    #[must_use]
    pub const fn diagnostic_code(self) -> &'static str {
        match self {
            Self::InvalidLimits => "STL_INVALID_LIMITS",
            Self::InputLimitExceeded => "STL_INPUT_LIMIT_EXCEEDED",
            Self::InvalidText => "STL_INVALID_TEXT",
            Self::InvalidStructure => "STL_INVALID_STRUCTURE",
            Self::InvalidNumber => "STL_INVALID_NUMBER",
            Self::UnsupportedAttributeData => "STL_UNSUPPORTED_ATTRIBUTE_DATA",
            Self::TriangleLimitExceeded => "STL_TRIANGLE_LIMIT_EXCEEDED",
            Self::DegenerateTriangle => "STL_DEGENERATE_TRIANGLE",
            Self::EmptyMesh => "STL_EMPTY_MESH",
        }
    }
}

impl fmt::Display for AsciiStlError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.diagnostic_code())
    }
}

impl Error for AsciiStlError {}

/// Encoding-neutral name for bounded STL analysis failures.
pub type StlError = AsciiStlError;

#[derive(Clone, Copy, Debug)]
struct Triangle([MeshVector3; 3]);

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct VertexKey([u64; 3]);

impl From<MeshVector3> for VertexKey {
    fn from(vertex: MeshVector3) -> Self {
        Self([
            canonical_float_bits(vertex.x),
            canonical_float_bits(vertex.y),
            canonical_float_bits(vertex.z),
        ])
    }
}

fn canonical_float_bits(value: f64) -> u64 {
    (if value == 0.0 { 0.0 } else { value }).to_bits()
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct EdgeKey(VertexKey, VertexKey);

#[derive(Clone, Copy, Debug, Default)]
struct EdgeUses {
    forward: usize,
    reverse: usize,
}

impl EdgeUses {
    const fn count(self) -> usize {
        self.forward + self.reverse
    }
}

/// Parses and measures one ASCII STL byte stream without resolving a source path.
pub fn analyze_ascii_stl(bytes: &[u8], limits: StlLimits) -> Result<StlMeshEvidence, StlError> {
    if bytes.len() > limits.max_input_bytes {
        return Err(AsciiStlError::InputLimitExceeded);
    }
    let text = std::str::from_utf8(bytes).map_err(|_| AsciiStlError::InvalidText)?;
    let mut lines = text.lines().map(str::trim).filter(|line| !line.is_empty());
    let header = lines.next().ok_or(AsciiStlError::EmptyMesh)?;
    require_keyword_with_optional_name(header, "solid")?;

    let mut triangles = Vec::new();
    loop {
        let line = lines.next().ok_or(AsciiStlError::InvalidStructure)?;
        if keyword_matches(line, "endsolid") {
            if lines.next().is_some() {
                return Err(AsciiStlError::InvalidStructure);
            }
            break;
        }
        parse_facet_normal(line)?;
        require_exact_line(lines.next(), "outer loop")?;
        let vertices = [
            parse_vertex(lines.next())?,
            parse_vertex(lines.next())?,
            parse_vertex(lines.next())?,
        ];
        require_exact_line(lines.next(), "endloop")?;
        require_exact_line(lines.next(), "endfacet")?;
        if triangles.len() == limits.max_triangles {
            return Err(AsciiStlError::TriangleLimitExceeded);
        }
        validate_triangle(vertices)?;
        triangles.push(Triangle(vertices));
    }
    if triangles.is_empty() {
        return Err(AsciiStlError::EmptyMesh);
    }

    analyze_triangles(&triangles, ASCII_STL_ANALYZER_VERSION, StlEncoding::Ascii)
}

/// Detects ASCII versus binary STL from content framing and measures it.
pub fn analyze_stl(bytes: &[u8], limits: StlLimits) -> Result<StlMeshEvidence, StlError> {
    if bytes.len() > limits.max_input_bytes {
        return Err(AsciiStlError::InputLimitExceeded);
    }
    if has_exact_binary_framing(bytes) {
        analyze_binary_stl(bytes, limits)
    } else {
        analyze_ascii_stl(bytes, limits)
    }
}

/// Parses and measures one binary STL byte stream without resolving a source path.
pub fn analyze_binary_stl(bytes: &[u8], limits: StlLimits) -> Result<StlMeshEvidence, StlError> {
    if bytes.len() > limits.max_input_bytes {
        return Err(AsciiStlError::InputLimitExceeded);
    }
    if bytes.len() < BINARY_HEADER_BYTES + BINARY_COUNT_BYTES {
        return Err(AsciiStlError::InvalidStructure);
    }

    let triangle_count = read_u32(bytes, BINARY_HEADER_BYTES)? as usize;
    if triangle_count == 0 {
        return Err(AsciiStlError::EmptyMesh);
    }
    if triangle_count > limits.max_triangles {
        return Err(AsciiStlError::TriangleLimitExceeded);
    }
    let expected_bytes = binary_length(triangle_count).ok_or(AsciiStlError::InvalidStructure)?;
    if bytes.len() != expected_bytes {
        return Err(AsciiStlError::InvalidStructure);
    }

    let mut triangles = Vec::with_capacity(triangle_count);
    let mut offset = BINARY_HEADER_BYTES + BINARY_COUNT_BYTES;
    for _ in 0..triangle_count {
        for _ in 0..3 {
            let normal_component = read_f32(bytes, offset)?;
            if !normal_component.is_finite() {
                return Err(AsciiStlError::InvalidNumber);
            }
            offset += 4;
        }
        let mut vertices = [MeshVector3::new(0.0, 0.0, 0.0); 3];
        for vertex in &mut vertices {
            *vertex = MeshVector3::new(
                f64::from(read_f32(bytes, offset)?),
                f64::from(read_f32(bytes, offset + 4)?),
                f64::from(read_f32(bytes, offset + 8)?),
            );
            if !vertex.is_finite() {
                return Err(AsciiStlError::InvalidNumber);
            }
            offset += 12;
        }
        let attribute_bytes = read_u16(bytes, offset)?;
        offset += 2;
        if attribute_bytes != 0 {
            return Err(AsciiStlError::UnsupportedAttributeData);
        }
        validate_triangle(vertices)?;
        triangles.push(Triangle(vertices));
    }

    analyze_triangles(&triangles, BINARY_STL_ANALYZER_VERSION, StlEncoding::Binary)
}

fn has_exact_binary_framing(bytes: &[u8]) -> bool {
    if bytes.len() < BINARY_HEADER_BYTES + BINARY_COUNT_BYTES {
        return false;
    }
    read_u32(bytes, BINARY_HEADER_BYTES)
        .ok()
        .and_then(|count| usize::try_from(count).ok())
        .and_then(binary_length)
        == Some(bytes.len())
}

fn binary_length(triangle_count: usize) -> Option<usize> {
    triangle_count
        .checked_mul(BINARY_TRIANGLE_BYTES)?
        .checked_add(BINARY_HEADER_BYTES + BINARY_COUNT_BYTES)
}

fn read_u16(bytes: &[u8], offset: usize) -> Result<u16, AsciiStlError> {
    let value = bytes
        .get(offset..offset + 2)
        .ok_or(AsciiStlError::InvalidStructure)?;
    Ok(u16::from_le_bytes([value[0], value[1]]))
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, AsciiStlError> {
    let value = bytes
        .get(offset..offset + 4)
        .ok_or(AsciiStlError::InvalidStructure)?;
    Ok(u32::from_le_bytes([value[0], value[1], value[2], value[3]]))
}

fn read_f32(bytes: &[u8], offset: usize) -> Result<f32, AsciiStlError> {
    let bits = read_u32(bytes, offset)?;
    Ok(f32::from_bits(bits))
}

fn validate_triangle(vertices: [MeshVector3; 3]) -> Result<(), AsciiStlError> {
    let double_area = vertices[1]
        .subtract(vertices[0])
        .cross(vertices[2].subtract(vertices[0]))
        .norm();
    if !double_area.is_finite() || double_area <= DEGENERATE_AREA_EPSILON {
        Err(AsciiStlError::DegenerateTriangle)
    } else {
        Ok(())
    }
}

fn require_keyword_with_optional_name(line: &str, keyword: &str) -> Result<(), AsciiStlError> {
    if keyword_matches(line, keyword) {
        Ok(())
    } else {
        Err(AsciiStlError::InvalidStructure)
    }
}

fn keyword_matches(line: &str, keyword: &str) -> bool {
    line == keyword
        || line
            .strip_prefix(keyword)
            .and_then(|remainder| remainder.chars().next())
            .is_some_and(char::is_whitespace)
}

fn require_exact_line(line: Option<&str>, expected: &str) -> Result<(), AsciiStlError> {
    if line == Some(expected) {
        Ok(())
    } else {
        Err(AsciiStlError::InvalidStructure)
    }
}

fn parse_facet_normal(line: &str) -> Result<(), AsciiStlError> {
    let tokens: Vec<_> = line.split_whitespace().collect();
    if tokens.len() != 5 || tokens[0] != "facet" || tokens[1] != "normal" {
        return Err(AsciiStlError::InvalidStructure);
    }
    for token in &tokens[2..] {
        parse_number(token)?;
    }
    Ok(())
}

fn parse_vertex(line: Option<&str>) -> Result<MeshVector3, AsciiStlError> {
    let tokens: Vec<_> = line
        .ok_or(AsciiStlError::InvalidStructure)?
        .split_whitespace()
        .collect();
    if tokens.len() != 4 || tokens[0] != "vertex" {
        return Err(AsciiStlError::InvalidStructure);
    }
    Ok(MeshVector3::new(
        parse_number(tokens[1])?,
        parse_number(tokens[2])?,
        parse_number(tokens[3])?,
    ))
}

fn parse_number(token: &str) -> Result<f64, AsciiStlError> {
    let value = token
        .parse::<f64>()
        .map_err(|_| AsciiStlError::InvalidNumber)?;
    if value.is_finite() {
        Ok(value)
    } else {
        Err(AsciiStlError::InvalidNumber)
    }
}

fn analyze_triangles(
    triangles: &[Triangle],
    algorithm_version: &'static str,
    encoding: StlEncoding,
) -> Result<AsciiStlMeshEvidence, AsciiStlError> {
    let mut minimum = MeshVector3::new(f64::INFINITY, f64::INFINITY, f64::INFINITY);
    let mut maximum = MeshVector3::new(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY);
    let mut area = 0.0;
    let mut determinant_sum = 0.0;
    let mut centroid_numerator = MeshVector3::new(0.0, 0.0, 0.0);
    let mut edges = BTreeMap::<EdgeKey, EdgeUses>::new();

    for triangle in triangles {
        let [a, b, c] = triangle.0;
        for vertex in [a, b, c] {
            minimum.x = minimum.x.min(vertex.x);
            minimum.y = minimum.y.min(vertex.y);
            minimum.z = minimum.z.min(vertex.z);
            maximum.x = maximum.x.max(vertex.x);
            maximum.y = maximum.y.max(vertex.y);
            maximum.z = maximum.z.max(vertex.z);
        }
        area += b.subtract(a).cross(c.subtract(a)).norm() * 0.5;
        let determinant = a.dot(b.cross(c));
        determinant_sum += determinant;
        centroid_numerator = centroid_numerator.add(a.add(b).add(c).scale(determinant));
        for (from, to) in [(a, b), (b, c), (c, a)] {
            let from = VertexKey::from(from);
            let to = VertexKey::from(to);
            let (key, forward) = if from < to {
                (EdgeKey(from, to), true)
            } else {
                (EdgeKey(to, from), false)
            };
            let uses = edges.entry(key).or_default();
            if forward {
                uses.forward += 1;
            } else {
                uses.reverse += 1;
            }
        }
    }

    let manifold = edges.values().all(|uses| uses.count() <= 2);
    let watertight = manifold && edges.values().all(|uses| uses.count() == 2);
    let consistently_wound = watertight
        && edges
            .values()
            .all(|uses| uses.forward == 1 && uses.reverse == 1);
    let extents = maximum.subtract(minimum);
    if !extents.is_finite()
        || !area.is_finite()
        || !determinant_sum.is_finite()
        || !centroid_numerator.is_finite()
    {
        return Err(AsciiStlError::InvalidNumber);
    }
    let (enclosed_volume, center_of_mass) =
        if consistently_wound && determinant_sum.abs() > ZERO_VOLUME_EPSILON {
            (
                Some((determinant_sum / 6.0).abs()),
                Some(centroid_numerator.scale(1.0 / (4.0 * determinant_sum))),
            )
        } else {
            (None, None)
        };

    let mut warnings = vec![
        warning("UNITS_MISSING_REQUIRES_CONFIRMATION"),
        warning("MESH_NOT_EXACT_BREP"),
    ];
    if !manifold {
        warnings.push(warning("NON_MANIFOLD_EDGE"));
    }
    if !watertight {
        warnings.push(warning("OPEN_BOUNDARY"));
    } else if !consistently_wound {
        warnings.push(warning("INCONSISTENT_WINDING"));
    } else if enclosed_volume.is_none() {
        warnings.push(warning("ZERO_ENCLOSED_VOLUME"));
    }
    if enclosed_volume.is_none() {
        warnings.push(warning("CLOSED_VOLUME_UNAVAILABLE"));
    }

    Ok(AsciiStlMeshEvidence {
        algorithm_version,
        encoding,
        detected_format: ModelFormat::Stl,
        representation: RepresentationBasis::Mesh,
        source_units: ModelLengthUnit::Unknown,
        unit_resolution: UnitResolutionMethod::Unresolved,
        triangle_count: triangles.len(),
        manifold,
        watertight,
        consistently_wound,
        aabb_extents_source_units: extents,
        surface_area_source_units_squared: area,
        enclosed_volume_source_units_cubed: enclosed_volume,
        center_of_mass_source_units: center_of_mass,
        warnings,
    })
}

fn warning(code: &str) -> GeometryWarningCode {
    GeometryWarningCode::new(code).expect("static ASCII STL warning code must be valid")
}
