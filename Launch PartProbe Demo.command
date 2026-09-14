#!/bin/zsh

set -eu

launcher_directory=${0:A:h}
exec "${launcher_directory}/scripts/launch_macos_demo.zsh"
