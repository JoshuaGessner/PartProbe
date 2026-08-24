//! Format-neutral deterministic triangle-mesh measurements.

use std::collections::BTreeMap;

use partprobe_geometry_core::{
    GeometryConfidence, GeometryConfidenceLevel, GeometryConfidenceReasonCode, GeometryWarningCode,
};
use serde::Serialize;

const DEGENERATE_AREA_EPSILON: f64 = 1.0e-12;
const ZERO_VOLUME_EPSILON: f64 = 1.0e-12;

/// Versioned identity for the bounded exact-predicate triangle-pair comparison.
pub const MESH_SELF_INTERSECTION_ALGORITHM_VERSION: &str =
    "partprobe-exact-mesh-intersection-spike-v1";
/// Versioned identity for categorical mesh confidence and reason selection.
pub const MESH_CONFIDENCE_POLICY_VERSION: &str = "partprobe-mesh-confidence-policy-v1";

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

/// Result of the bounded exact-predicate mesh self-intersection comparison.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MeshSelfIntersectionState {
    /// No intersection was found under the governed exact-predicate policy.
    NotDetected,
    /// At least one non-topological triangle intersection was found.
    Detected,
    /// A coplanar overlapping-bounds pair needs a future tolerance policy.
    Indeterminate,
}

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
    pub(crate) self_intersection: MeshSelfIntersectionState,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TrianglePairIntersection {
    None,
    Detected,
    Indeterminate,
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
    let self_intersection = analyze_self_intersections(triangles);
    let extents = maximum.subtract(minimum);
    if !extents.is_finite()
        || !area.is_finite()
        || !determinant_sum.is_finite()
        || !centroid_numerator.is_finite()
    {
        return Err(MeshAnalysisError::InvalidNumber);
    }
    let (enclosed_volume, center_of_mass) = if consistently_wound
        && self_intersection == MeshSelfIntersectionState::NotDetected
        && determinant_sum.abs() > ZERO_VOLUME_EPSILON
    {
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
        self_intersection,
        aabb_extents: extents,
        surface_area: area,
        enclosed_volume,
        center_of_mass,
    })
}

fn analyze_self_intersections(triangles: &[Triangle]) -> MeshSelfIntersectionState {
    let mut indeterminate = false;
    for (left_index, left) in triangles.iter().enumerate() {
        for right in &triangles[left_index + 1..] {
            match triangle_pair_intersection(*left, *right) {
                TrianglePairIntersection::Detected => {
                    return MeshSelfIntersectionState::Detected;
                }
                TrianglePairIntersection::Indeterminate => indeterminate = true,
                TrianglePairIntersection::None => {}
            }
        }
    }
    if indeterminate {
        MeshSelfIntersectionState::Indeterminate
    } else {
        MeshSelfIntersectionState::NotDetected
    }
}

fn triangle_pair_intersection(left: Triangle, right: Triangle) -> TrianglePairIntersection {
    if !aabbs_overlap(left, right) {
        return TrianglePairIntersection::None;
    }

    let shared_vertices: Vec<_> = left
        .0
        .into_iter()
        .filter(|vertex| {
            right
                .0
                .into_iter()
                .map(VertexKey::from)
                .any(|other| other == VertexKey::from(*vertex))
        })
        .collect();
    if shared_vertices.len() == 3 {
        return TrianglePairIntersection::Indeterminate;
    }

    let left_normal = left.0[1]
        .subtract(left.0[0])
        .cross(left.0[2].subtract(left.0[0]));
    let right_normal = right.0[1]
        .subtract(right.0[0])
        .cross(right.0[2].subtract(right.0[0]));
    let normals_are_parallel = left_normal.cross(right_normal).norm() == 0.0;
    let coplanar = right.0[0].subtract(left.0[0]).dot(left_normal) == 0.0;
    if shared_vertices.len() == 2 {
        if normals_are_parallel && coplanar {
            let shared_start = shared_vertices[0];
            let shared_edge = shared_vertices[1].subtract(shared_start);
            let left_opposite = left
                .0
                .into_iter()
                .find(|vertex| {
                    !shared_vertices
                        .iter()
                        .any(|shared| VertexKey::from(*shared) == VertexKey::from(*vertex))
                })
                .expect("a nondegenerate triangle sharing one edge has one opposite vertex");
            let right_opposite = right
                .0
                .into_iter()
                .find(|vertex| {
                    !shared_vertices
                        .iter()
                        .any(|shared| VertexKey::from(*shared) == VertexKey::from(*vertex))
                })
                .expect("a nondegenerate triangle sharing one edge has one opposite vertex");
            let left_side = shared_edge
                .cross(left_opposite.subtract(shared_start))
                .dot(left_normal);
            let right_side = shared_edge
                .cross(right_opposite.subtract(shared_start))
                .dot(left_normal);
            if left_side == 0.0 || right_side == 0.0 || left_side.signum() == right_side.signum() {
                return TrianglePairIntersection::Indeterminate;
            }
        }
        return TrianglePairIntersection::None;
    }
    if normals_are_parallel {
        return if coplanar {
            TrianglePairIntersection::Indeterminate
        } else {
            TrianglePairIntersection::None
        };
    }

    for (segment, triangle) in [(left, right), (right, left)] {
        for [start, end] in triangle_edges(segment) {
            if let Some(point) = segment_triangle_intersection(start, end, triangle)
                && !shared_vertices
                    .iter()
                    .any(|shared| VertexKey::from(*shared) == VertexKey::from(point))
            {
                return TrianglePairIntersection::Detected;
            }
        }
    }
    TrianglePairIntersection::None
}

fn aabbs_overlap(left: Triangle, right: Triangle) -> bool {
    let (left_minimum, left_maximum) = triangle_bounds(left);
    let (right_minimum, right_maximum) = triangle_bounds(right);
    left_minimum.x <= right_maximum.x
        && right_minimum.x <= left_maximum.x
        && left_minimum.y <= right_maximum.y
        && right_minimum.y <= left_maximum.y
        && left_minimum.z <= right_maximum.z
        && right_minimum.z <= left_maximum.z
}

fn triangle_bounds(triangle: Triangle) -> (MeshVector3, MeshVector3) {
    let mut minimum = triangle.0[0];
    let mut maximum = triangle.0[0];
    for vertex in triangle.0[1..].iter().copied() {
        minimum.x = minimum.x.min(vertex.x);
        minimum.y = minimum.y.min(vertex.y);
        minimum.z = minimum.z.min(vertex.z);
        maximum.x = maximum.x.max(vertex.x);
        maximum.y = maximum.y.max(vertex.y);
        maximum.z = maximum.z.max(vertex.z);
    }
    (minimum, maximum)
}

fn triangle_edges(triangle: Triangle) -> [[MeshVector3; 2]; 3] {
    let [a, b, c] = triangle.0;
    [[a, b], [b, c], [c, a]]
}

fn segment_triangle_intersection(
    start: MeshVector3,
    end: MeshVector3,
    triangle: Triangle,
) -> Option<MeshVector3> {
    let [a, b, c] = triangle.0;
    let direction = end.subtract(start);
    let edge_one = b.subtract(a);
    let edge_two = c.subtract(a);
    let cross = direction.cross(edge_two);
    let determinant = edge_one.dot(cross);
    if determinant == 0.0 {
        return None;
    }
    let inverse_determinant = 1.0 / determinant;
    let from_a = start.subtract(a);
    let u = from_a.dot(cross) * inverse_determinant;
    if !(0.0..=1.0).contains(&u) {
        return None;
    }
    let second_cross = from_a.cross(edge_one);
    let v = direction.dot(second_cross) * inverse_determinant;
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    let distance = edge_two.dot(second_cross) * inverse_determinant;
    if !(0.0..=1.0).contains(&distance) {
        return None;
    }
    Some(start.add(direction.scale(distance)))
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
    } else if analysis.self_intersection == MeshSelfIntersectionState::NotDetected
        && analysis.enclosed_volume.is_none()
    {
        warnings.push(warning("ZERO_ENCLOSED_VOLUME"));
    }
    match analysis.self_intersection {
        MeshSelfIntersectionState::Detected => {
            warnings.push(warning("SELF_INTERSECTION_DETECTED"));
        }
        MeshSelfIntersectionState::Indeterminate => {
            warnings.push(warning("SELF_INTERSECTION_INDETERMINATE"));
        }
        MeshSelfIntersectionState::NotDetected => {}
    }
    if analysis.enclosed_volume.is_none() {
        warnings.push(warning("CLOSED_VOLUME_UNAVAILABLE"));
    }
    warnings
}

pub(crate) fn mesh_confidence(analysis: &MeshAnalysis, units_resolved: bool) -> GeometryConfidence {
    let mut reasons = vec![confidence_reason("MESH_REPRESENTATION_CEILING")];
    if !units_resolved {
        reasons.push(confidence_reason("UNITS_UNRESOLVED"));
    }
    if !analysis.manifold {
        reasons.push(confidence_reason("NON_MANIFOLD_EDGE"));
    }
    if !analysis.watertight {
        reasons.push(confidence_reason("OPEN_BOUNDARY"));
    } else if !analysis.consistently_wound {
        reasons.push(confidence_reason("INCONSISTENT_WINDING"));
    }
    match analysis.self_intersection {
        MeshSelfIntersectionState::Detected => {
            reasons.push(confidence_reason("SELF_INTERSECTION_DETECTED"));
        }
        MeshSelfIntersectionState::Indeterminate => {
            reasons.push(confidence_reason("SELF_INTERSECTION_INDETERMINATE"));
        }
        MeshSelfIntersectionState::NotDetected => {}
    }
    let level = if units_resolved
        && analysis.manifold
        && analysis.watertight
        && analysis.consistently_wound
        && analysis.self_intersection == MeshSelfIntersectionState::NotDetected
    {
        GeometryConfidenceLevel::Low
    } else {
        GeometryConfidenceLevel::NeedsReview
    };
    GeometryConfidence::new(level, reasons).expect("static mesh confidence must be valid")
}

fn warning(code: &str) -> GeometryWarningCode {
    GeometryWarningCode::new(code).expect("static mesh-analysis warning code must be valid")
}

fn confidence_reason(code: &str) -> GeometryConfidenceReasonCode {
    GeometryConfidenceReasonCode::new(code).expect("static mesh confidence reason must be valid")
}
