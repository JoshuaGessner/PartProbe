//! Format-neutral deterministic triangle-mesh measurements.

use std::collections::BTreeMap;

use partprobe_geometry_core::GeometryWarningCode;
use serde::Serialize;

const DEGENERATE_AREA_EPSILON: f64 = 1.0e-12;
const ZERO_VOLUME_EPSILON: f64 = 1.0e-12;

/// Three-dimensional mesh coordinate or measurement vector.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct MeshVector3 {
    pub(crate) x: f64,
    pub(crate) y: f64,
    pub(crate) z: f64,
}

impl MeshVector3 {
    pub(crate) const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    /// Returns the vector components.
    #[must_use]
    pub const fn components(self) -> [f64; 3] {
        [self.x, self.y, self.z]
    }

    pub(crate) fn subtract(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }

    fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }

    pub(crate) fn scale(self, factor: f64) -> Self {
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

    pub(crate) fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.z.is_finite()
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Triangle(pub(crate) [MeshVector3; 3]);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MeshAnalysisError {
    DegenerateTriangle,
    InvalidNumber,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct MeshAnalysis {
    pub(crate) manifold: bool,
    pub(crate) watertight: bool,
    pub(crate) consistently_wound: bool,
    pub(crate) aabb_extents: MeshVector3,
    pub(crate) surface_area: f64,
    pub(crate) enclosed_volume: Option<f64>,
    pub(crate) center_of_mass: Option<MeshVector3>,
}

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

pub(crate) fn validate_triangle(triangle: Triangle) -> Result<(), MeshAnalysisError> {
    let [a, b, c] = triangle.0;
    let double_area = b.subtract(a).cross(c.subtract(a)).norm();
    if !double_area.is_finite() || double_area <= DEGENERATE_AREA_EPSILON {
        Err(MeshAnalysisError::DegenerateTriangle)
    } else {
        Ok(())
    }
}

pub(crate) fn analyze_triangles(triangles: &[Triangle]) -> Result<MeshAnalysis, MeshAnalysisError> {
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
        return Err(MeshAnalysisError::InvalidNumber);
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

    Ok(MeshAnalysis {
        manifold,
        watertight,
        consistently_wound,
        aabb_extents: extents,
        surface_area: area,
        enclosed_volume,
        center_of_mass,
    })
}

pub(crate) fn mesh_warning_codes(analysis: &MeshAnalysis) -> Vec<GeometryWarningCode> {
    let mut warnings = vec![warning("MESH_NOT_EXACT_BREP")];
    if !analysis.manifold {
        warnings.push(warning("NON_MANIFOLD_EDGE"));
    }
    if !analysis.watertight {
        warnings.push(warning("OPEN_BOUNDARY"));
    } else if !analysis.consistently_wound {
        warnings.push(warning("INCONSISTENT_WINDING"));
    } else if analysis.enclosed_volume.is_none() {
        warnings.push(warning("ZERO_ENCLOSED_VOLUME"));
    }
    if analysis.enclosed_volume.is_none() {
        warnings.push(warning("CLOSED_VOLUME_UNAVAILABLE"));
    }
    warnings
}

fn warning(code: &str) -> GeometryWarningCode {
    GeometryWarningCode::new(code).expect("static mesh-analysis warning code must be valid")
}
