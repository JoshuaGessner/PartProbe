#include <BRepBndLib.hxx>
#include <BRepGProp.hxx>
#include <BRepMesh_IncrementalMesh.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRep_Tool.hxx>
#include <Bnd_Box.hxx>
#include <GProp_GProps.hxx>
#include <IFSelect_ReturnStatus.hxx>
#include <Message_ProgressIndicator.hxx>
#include <STEPControl_Reader.hxx>
#include <STEPControl_StepModelType.hxx>
#include <STEPControl_Writer.hxx>
#include <Standard_Failure.hxx>
#include <TopAbs_Orientation.hxx>
#include <TopAbs_ShapeEnum.hxx>
#include <TopExp_Explorer.hxx>
#include <TopLoc_Location.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Shape.hxx>
#include <gp_Pnt.hxx>
#include <gp_Trsf.hxx>

#include <cstddef>
#include <cstdint>
#include <cmath>
#include <cstring>
#include <limits>
#include <memory>
#include <new>
#include <sstream>
#include <string>
#include <utility>
#include <vector>

namespace {
constexpr std::uint32_t kAbiVersion = 4;
constexpr std::uint32_t kDisplayTessellationAbiVersion = 1;
constexpr std::size_t kDiagnosticCapacity = 64;
constexpr char kStepStreamName[] = "partprobe-input.step";
using CancellationProbe = std::uint8_t (*)(const void *);

struct NativeResult {
  std::uint32_t abi_version;
  std::uint64_t transferred_roots;
  std::uint64_t solid_body_count;
  double surface_area_mm2;
  double enclosed_volume_mm3;
  double center_of_mass_x_mm;
  double center_of_mass_y_mm;
  double center_of_mass_z_mm;
  double aabb_extent_x_mm;
  double aabb_extent_y_mm;
  double aabb_extent_z_mm;
  char diagnostic_code[kDiagnosticCapacity];
};

struct NativeDisplayResult {
  std::uint32_t abi_version;
  std::uint32_t reserved;
  std::uint64_t vertex_count;
  std::uint64_t triangle_count;
  const float *positions;
  const std::uint32_t *triangle_indices;
  void *ownership;
  char diagnostic_code[kDiagnosticCapacity];
};

struct DisplayMeshStorage {
  std::vector<float> positions;
  std::vector<std::uint32_t> triangle_indices;
};

void set_diagnostic(NativeResult *result, const char *code) noexcept {
  std::size_t length = 0;
  while (length + 1 < kDiagnosticCapacity && code[length] != '\0') {
    result->diagnostic_code[length] = code[length];
    ++length;
  }
  result->diagnostic_code[length] = '\0';
}

int fail(NativeResult *result, const char *code) noexcept {
  set_diagnostic(result, code);
  return 1;
}

void set_display_diagnostic(NativeDisplayResult *result,
                            const char *code) noexcept {
  std::size_t length = 0;
  while (length + 1 < kDiagnosticCapacity && code[length] != '\0') {
    result->diagnostic_code[length] = code[length];
    ++length;
  }
  result->diagnostic_code[length] = '\0';
}

int fail_display(NativeDisplayResult *result, const char *code) noexcept {
  set_display_diagnostic(result, code);
  return 1;
}

class PartProbeProgressIndicator final : public Message_ProgressIndicator {
public:
  PartProbeProgressIndicator(CancellationProbe probe, const void *context)
      : probe_(probe), context_(context) {}

  bool UserBreak() override {
    return probe_ != nullptr && probe_(context_) != 0;
  }

  void Show(const Message_ProgressScope &, const bool) override {}

private:
  CancellationProbe probe_;
  const void *context_;
};

bool cancellation_requested(CancellationProbe probe,
                            const void *context) noexcept {
  return probe != nullptr && probe(context) != 0;
}

const char *transfer_shape(STEPControl_Reader &reader, TopoDS_Shape &shape,
                           std::uint64_t &transferred_roots,
                           CancellationProbe cancel_probe,
                           const void *cancel_context) {
  if (cancellation_requested(cancel_probe, cancel_context)) {
    return "OCCT_CANCELLED";
  }
  reader.SetSystemLengthUnit(1.0);
  occ::handle<Message_ProgressIndicator> progress =
      new PartProbeProgressIndicator(cancel_probe, cancel_context);
  const int transferred = reader.TransferRoots(progress->Start());
  if (cancellation_requested(cancel_probe, cancel_context)) {
    return "OCCT_CANCELLED";
  }
  if (transferred <= 0) {
    return "STEP_TRANSFER_FAILED";
  }
  shape = reader.OneShape();
  if (shape.IsNull()) {
    return "STEP_NO_SHAPE";
  }
  transferred_roots = static_cast<std::uint64_t>(transferred);
  return nullptr;
}

int transfer_and_measure(STEPControl_Reader &reader, NativeResult *result,
                         CancellationProbe cancel_probe,
                         const void *cancel_context) {
  TopoDS_Shape shape;
  std::uint64_t transferred = 0;
  if (const char *code = transfer_shape(reader, shape, transferred, cancel_probe,
                                        cancel_context)) {
    return fail(result, code);
  }

  if (cancellation_requested(cancel_probe, cancel_context)) {
    return fail(result, "OCCT_CANCELLED");
  }
  result->transferred_roots = transferred;
  for (TopExp_Explorer explorer(shape, TopAbs_SOLID); explorer.More();
       explorer.Next()) {
    ++result->solid_body_count;
  }

  if (cancellation_requested(cancel_probe, cancel_context)) {
    return fail(result, "OCCT_CANCELLED");
  }
  Bnd_Box bounds;
  BRepBndLib::AddOptimal(shape, bounds, false, false);
  if (bounds.IsVoid() || bounds.IsOpen()) {
    return fail(result, "OCCT_INVALID_BOUNDS");
  }
  double x_min = 0.0;
  double y_min = 0.0;
  double z_min = 0.0;
  double x_max = 0.0;
  double y_max = 0.0;
  double z_max = 0.0;
  bounds.Get(x_min, y_min, z_min, x_max, y_max, z_max);
  result->aabb_extent_x_mm = x_max - x_min;
  result->aabb_extent_y_mm = y_max - y_min;
  result->aabb_extent_z_mm = z_max - z_min;
  if (!std::isfinite(result->aabb_extent_x_mm) ||
      !std::isfinite(result->aabb_extent_y_mm) ||
      !std::isfinite(result->aabb_extent_z_mm) ||
      !(result->aabb_extent_x_mm > 0.0) ||
      !(result->aabb_extent_y_mm > 0.0) ||
      !(result->aabb_extent_z_mm > 0.0)) {
    return fail(result, "OCCT_INVALID_BOUNDS");
  }

  if (cancellation_requested(cancel_probe, cancel_context)) {
    return fail(result, "OCCT_CANCELLED");
  }
  GProp_GProps surface;
  BRepGProp::SurfaceProperties(shape, surface);
  if (cancellation_requested(cancel_probe, cancel_context)) {
    return fail(result, "OCCT_CANCELLED");
  }
  result->surface_area_mm2 = surface.Mass();

  if (result->solid_body_count > 0) {
    GProp_GProps volume;
    BRepGProp::VolumeProperties(shape, volume);
    if (cancellation_requested(cancel_probe, cancel_context)) {
      return fail(result, "OCCT_CANCELLED");
    }
    const gp_Pnt center = volume.CentreOfMass();
    result->enclosed_volume_mm3 = volume.Mass();
    result->center_of_mass_x_mm = center.X();
    result->center_of_mass_y_mm = center.Y();
    result->center_of_mass_z_mm = center.Z();
  }
  return 0;
}

bool checked_display_totals(std::uint64_t current_vertices,
                            std::uint64_t current_triangles,
                            std::uint64_t added_vertices,
                            std::uint64_t added_triangles,
                            std::uint64_t max_vertices,
                            std::uint64_t max_triangles,
                            std::uint64_t max_bytes,
                            std::uint64_t &new_vertices,
                            std::uint64_t &new_triangles) noexcept {
  if (current_vertices > max_vertices || current_triangles > max_triangles ||
      added_vertices > max_vertices - current_vertices ||
      added_triangles > max_triangles - current_triangles) {
    return false;
  }
  new_vertices = current_vertices + added_vertices;
  new_triangles = current_triangles + added_triangles;
  if (new_vertices > std::numeric_limits<std::uint32_t>::max() ||
      new_vertices > std::numeric_limits<std::uint64_t>::max() / 12 ||
      new_triangles > std::numeric_limits<std::uint64_t>::max() / 12 ||
      new_vertices > std::numeric_limits<std::size_t>::max() / 3 ||
      new_triangles > std::numeric_limits<std::size_t>::max() / 3) {
    return false;
  }
  const std::uint64_t vertex_bytes = new_vertices * 12;
  const std::uint64_t index_bytes = new_triangles * 12;
  return index_bytes <= max_bytes && vertex_bytes <= max_bytes - index_bytes;
}

int tessellate_shape(const TopoDS_Shape &shape, double linear_deflection_mm,
                     double angular_deflection_degrees,
                     std::uint64_t max_vertices,
                     std::uint64_t max_triangles, std::uint64_t max_bytes,
                     NativeDisplayResult *result,
                     CancellationProbe cancel_probe,
                     const void *cancel_context) {
  if (!std::isfinite(linear_deflection_mm) ||
      !(linear_deflection_mm > 0.0) ||
      !std::isfinite(angular_deflection_degrees) ||
      !(angular_deflection_degrees > 0.0)) {
    return fail_display(result, "OCCT_TESSELLATION_INVALID_PROFILE");
  }
  if (max_vertices == 0 || max_triangles == 0 || max_bytes == 0) {
    return fail_display(result, "OCCT_TESSELLATION_INVALID_LIMITS");
  }
  if (cancellation_requested(cancel_probe, cancel_context)) {
    return fail_display(result, "OCCT_CANCELLED");
  }

  constexpr double kPi = 3.141592653589793238462643383279502884;
  const double angular_deflection_radians =
      angular_deflection_degrees * kPi / 180.0;
  BRepMesh_IncrementalMesh mesher(shape, linear_deflection_mm, false,
                                  angular_deflection_radians, false);
  if (mesher.GetStatusFlags() != 0) {
    return fail_display(result, "OCCT_TESSELLATION_FAILED");
  }
  if (cancellation_requested(cancel_probe, cancel_context)) {
    return fail_display(result, "OCCT_CANCELLED");
  }

  auto storage = std::make_unique<DisplayMeshStorage>();
  std::uint64_t vertex_count = 0;
  std::uint64_t triangle_count = 0;
  for (TopExp_Explorer explorer(shape, TopAbs_FACE); explorer.More();
       explorer.Next()) {
    if (cancellation_requested(cancel_probe, cancel_context)) {
      return fail_display(result, "OCCT_CANCELLED");
    }
    const TopoDS_Face face = TopoDS::Face(explorer.Current());
    TopLoc_Location location;
    const occ::handle<Poly_Triangulation> &triangulation =
        BRep_Tool::Triangulation(face, location);
    if (triangulation.IsNull() || triangulation->NbNodes() <= 0 ||
        triangulation->NbTriangles() <= 0) {
      return fail_display(result, "OCCT_TESSELLATION_FAILED");
    }

    const std::uint64_t face_vertices =
        static_cast<std::uint64_t>(triangulation->NbNodes());
    const std::uint64_t face_triangles =
        static_cast<std::uint64_t>(triangulation->NbTriangles());
    std::uint64_t new_vertices = 0;
    std::uint64_t new_triangles = 0;
    if (!checked_display_totals(vertex_count, triangle_count, face_vertices,
                                face_triangles, max_vertices, max_triangles,
                                max_bytes, new_vertices, new_triangles)) {
      return fail_display(result, "OCCT_TESSELLATION_LIMIT_EXCEEDED");
    }

    const std::uint32_t base_index = static_cast<std::uint32_t>(vertex_count);
    const gp_Trsf transform = location.Transformation();
    storage->positions.reserve(static_cast<std::size_t>(new_vertices * 3));
    for (int node_index = 1; node_index <= triangulation->NbNodes();
         ++node_index) {
      const gp_Pnt point = triangulation->Node(node_index).Transformed(transform);
      const float x = static_cast<float>(point.X());
      const float y = static_cast<float>(point.Y());
      const float z = static_cast<float>(point.Z());
      if (!std::isfinite(x) || !std::isfinite(y) || !std::isfinite(z)) {
        return fail_display(result, "OCCT_TESSELLATION_FAILED");
      }
      storage->positions.push_back(x);
      storage->positions.push_back(y);
      storage->positions.push_back(z);
    }

    storage->triangle_indices.reserve(
        static_cast<std::size_t>(new_triangles * 3));
    const bool reverse = face.Orientation() == TopAbs_REVERSED;
    for (int triangle_index = 1;
         triangle_index <= triangulation->NbTriangles(); ++triangle_index) {
      int node_1 = 0;
      int node_2 = 0;
      int node_3 = 0;
      triangulation->Triangle(triangle_index).Get(node_1, node_2, node_3);
      if (node_1 <= 0 || node_2 <= 0 || node_3 <= 0 ||
          node_1 > triangulation->NbNodes() ||
          node_2 > triangulation->NbNodes() ||
          node_3 > triangulation->NbNodes()) {
        return fail_display(result, "OCCT_TESSELLATION_FAILED");
      }
      const std::uint32_t index_1 =
          base_index + static_cast<std::uint32_t>(node_1 - 1);
      const std::uint32_t index_2 =
          base_index + static_cast<std::uint32_t>(node_2 - 1);
      const std::uint32_t index_3 =
          base_index + static_cast<std::uint32_t>(node_3 - 1);
      storage->triangle_indices.push_back(index_1);
      storage->triangle_indices.push_back(reverse ? index_3 : index_2);
      storage->triangle_indices.push_back(reverse ? index_2 : index_3);
    }
    vertex_count = new_vertices;
    triangle_count = new_triangles;
  }

  if (vertex_count == 0 || triangle_count == 0) {
    return fail_display(result, "OCCT_TESSELLATION_EMPTY");
  }
  result->vertex_count = vertex_count;
  result->triangle_count = triangle_count;
  result->positions = storage->positions.data();
  result->triangle_indices = storage->triangle_indices.data();
  result->ownership = storage.release();
  return 0;
}
} // namespace

extern "C" std::uint32_t partprobe_occt_abi_version() noexcept {
  return kAbiVersion;
}

extern "C" int partprobe_occt_analyze_step_bytes(
    const std::uint8_t *bytes, std::size_t byte_count, NativeResult *result,
    std::size_t result_size, CancellationProbe cancel_probe,
    const void *cancel_context) noexcept {
  if (result == nullptr || result_size != sizeof(NativeResult)) {
    return 1;
  }
  std::memset(result, 0, sizeof(NativeResult));
  result->abi_version = kAbiVersion;
  if (bytes == nullptr || byte_count == 0) {
    return fail(result, "OCCT_INVALID_ARGUMENT");
  }

  try {
    if (cancellation_requested(cancel_probe, cancel_context)) {
      return fail(result, "OCCT_CANCELLED");
    }
    const std::string contents(reinterpret_cast<const char *>(bytes),
                               byte_count);
    std::istringstream stream(contents, std::ios::in | std::ios::binary);
    STEPControl_Reader reader;
    if (reader.ReadStream(kStepStreamName, stream) != IFSelect_RetDone) {
      return fail(result, "STEP_READ_FAILED");
    }
    return transfer_and_measure(reader, result, cancel_probe, cancel_context);
  } catch (const Standard_Failure &) {
    return fail(result, "OCCT_STANDARD_FAILURE");
  } catch (...) {
    return fail(result, "OCCT_UNKNOWN_FAILURE");
  }
}

extern "C" int partprobe_occt_analyze_step(const char *path,
                                            NativeResult *result,
                                            std::size_t result_size,
                                            CancellationProbe cancel_probe,
                                            const void *cancel_context) noexcept {
  if (result == nullptr || result_size != sizeof(NativeResult)) {
    return 1;
  }
  std::memset(result, 0, sizeof(NativeResult));
  result->abi_version = kAbiVersion;
  if (path == nullptr || path[0] == '\0') {
    return fail(result, "OCCT_INVALID_ARGUMENT");
  }

  try {
    STEPControl_Reader reader;
    if (reader.ReadFile(path) != IFSelect_RetDone) {
      return fail(result, "STEP_READ_FAILED");
    }
    return transfer_and_measure(reader, result, cancel_probe, cancel_context);
  } catch (const Standard_Failure &) {
    return fail(result, "OCCT_STANDARD_FAILURE");
  } catch (...) {
    return fail(result, "OCCT_UNKNOWN_FAILURE");
  }
}

extern "C" int partprobe_occt_tessellate_step_bytes(
    const std::uint8_t *bytes, std::size_t byte_count,
    double linear_deflection_mm, double angular_deflection_degrees,
    std::uint64_t max_vertices, std::uint64_t max_triangles,
    std::uint64_t max_bytes, NativeDisplayResult *result,
    std::size_t result_size, CancellationProbe cancel_probe,
    const void *cancel_context) noexcept {
  if (result == nullptr || result_size != sizeof(NativeDisplayResult)) {
    return 1;
  }
  std::memset(result, 0, sizeof(NativeDisplayResult));
  result->abi_version = kDisplayTessellationAbiVersion;
  if (bytes == nullptr || byte_count == 0) {
    return fail_display(result, "OCCT_INVALID_ARGUMENT");
  }

  try {
    if (cancellation_requested(cancel_probe, cancel_context)) {
      return fail_display(result, "OCCT_CANCELLED");
    }
    const std::string contents(reinterpret_cast<const char *>(bytes),
                               byte_count);
    std::istringstream stream(contents, std::ios::in | std::ios::binary);
    STEPControl_Reader reader;
    if (reader.ReadStream(kStepStreamName, stream) != IFSelect_RetDone) {
      return fail_display(result, "STEP_READ_FAILED");
    }
    TopoDS_Shape shape;
    std::uint64_t transferred_roots = 0;
    if (const char *code = transfer_shape(reader, shape, transferred_roots,
                                          cancel_probe, cancel_context)) {
      return fail_display(result, code);
    }
    return tessellate_shape(shape, linear_deflection_mm,
                            angular_deflection_degrees, max_vertices,
                            max_triangles, max_bytes, result, cancel_probe,
                            cancel_context);
  } catch (const std::bad_alloc &) {
    return fail_display(result, "OCCT_TESSELLATION_ALLOCATION_FAILED");
  } catch (const Standard_Failure &) {
    return fail_display(result, "OCCT_STANDARD_FAILURE");
  } catch (...) {
    return fail_display(result, "OCCT_UNKNOWN_FAILURE");
  }
}

extern "C" void partprobe_occt_free_display_mesh(void *ownership) noexcept {
  delete static_cast<DisplayMeshStorage *>(ownership);
}

extern "C" int partprobe_occt_write_step_cube(const char *path,
                                               double size_mm) noexcept {
  if (path == nullptr || path[0] == '\0' || !(size_mm > 0.0)) {
    return 1;
  }
  try {
    const TopoDS_Shape cube =
        BRepPrimAPI_MakeBox(size_mm, size_mm, size_mm).Shape();
    STEPControl_Writer writer;
    if (writer.Transfer(cube, STEPControl_AsIs) != IFSelect_RetDone) {
      return 1;
    }
    return writer.Write(path) == IFSelect_RetDone ? 0 : 1;
  } catch (const Standard_Failure &) {
    return 1;
  } catch (...) {
    return 1;
  }
}
