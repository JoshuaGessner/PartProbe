#include <BRepGProp.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <GProp_GProps.hxx>
#include <IFSelect_ReturnStatus.hxx>
#include <Message_ProgressIndicator.hxx>
#include <STEPControl_Reader.hxx>
#include <STEPControl_StepModelType.hxx>
#include <STEPControl_Writer.hxx>
#include <Standard_Failure.hxx>
#include <TopAbs_ShapeEnum.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS_Shape.hxx>
#include <gp_Pnt.hxx>

#include <cstddef>
#include <cstdint>
#include <cstring>
#include <sstream>
#include <string>

namespace {
constexpr std::uint32_t kAbiVersion = 3;
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
  char diagnostic_code[kDiagnosticCapacity];
};

void set_diagnostic(NativeResult *result, const char *code) noexcept {
  std::strncpy(result->diagnostic_code, code, kDiagnosticCapacity - 1);
  result->diagnostic_code[kDiagnosticCapacity - 1] = '\0';
}

int fail(NativeResult *result, const char *code) noexcept {
  set_diagnostic(result, code);
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

int transfer_and_measure(STEPControl_Reader &reader, NativeResult *result,
                         CancellationProbe cancel_probe,
                         const void *cancel_context) {
  if (cancellation_requested(cancel_probe, cancel_context)) {
    return fail(result, "OCCT_CANCELLED");
  }
  reader.SetSystemLengthUnit(1.0);
  occ::handle<Message_ProgressIndicator> progress =
      new PartProbeProgressIndicator(cancel_probe, cancel_context);
  const int transferred = reader.TransferRoots(progress->Start());
  if (cancellation_requested(cancel_probe, cancel_context)) {
    return fail(result, "OCCT_CANCELLED");
  }
  if (transferred <= 0) {
    return fail(result, "STEP_TRANSFER_FAILED");
  }
  const TopoDS_Shape shape = reader.OneShape();
  if (shape.IsNull()) {
    return fail(result, "STEP_NO_SHAPE");
  }

  if (cancellation_requested(cancel_probe, cancel_context)) {
    return fail(result, "OCCT_CANCELLED");
  }
  result->transferred_roots = static_cast<std::uint64_t>(transferred);
  for (TopExp_Explorer explorer(shape, TopAbs_SOLID); explorer.More();
       explorer.Next()) {
    ++result->solid_body_count;
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
