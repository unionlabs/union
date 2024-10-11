#!/bin/bash
set -o errexit -o nounset -o pipefail

build_gnu_x86_64.sh
build_gnu_aarch64.sh
