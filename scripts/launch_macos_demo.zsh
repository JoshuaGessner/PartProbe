#!/bin/zsh

set -eu

script_directory=${0:A:h}
repository_root=${script_directory:h}
application_bundle=${PARTPROBE_DEMO_APP:-${repository_root}/target/debug/bundle/macos/PartProbe.app}
application_binary=${application_bundle}/Contents/MacOS/partprobe-estimator-desktop
runtime_root=${application_bundle}/Contents/Resources/partprobe-native-runtime

if [[ ! -x ${application_binary} ]]; then
  print -u2 "PartProbe demo app is missing or not executable: ${application_binary}"
  print -u2 "Build the governed macOS package before launching the demo."
  exit 1
fi

python3 "${repository_root}/scripts/assemble_native_runtime.py" verify \
  --runtime-root "${runtime_root}"

print "Launching PartProbe from ${application_bundle}"
print "The packaged host will create and remove its private worker workspace."
"${application_binary}"
