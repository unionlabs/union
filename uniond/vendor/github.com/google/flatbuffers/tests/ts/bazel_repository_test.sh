#!/bin/bash

# This test makes sure that a separate repository can import the flatbuffers
# repository and use it in their JavaScript code.

# --- begin runfiles.bash initialization v3 ---
# Copy-pasted from the Bazel Bash runfiles library v3.
set -uo pipefail; set +e; f=bazel_tools/tools/bash/runfiles/runfiles.bash
source "${RUNFILES_DIR:-/dev/null}/$f" 2>/dev/null || \
  source "$(grep -sm1 "^$f " "${RUNFILES_MANIFEST_FILE:-/dev/null}" | cut -f2- -d' ')" 2>/dev/null || \
  source "$0.runfiles/$f" 2>/dev/null || \
  source "$(grep -sm1 "^$f " "$0.runfiles_manifest" | cut -f2- -d' ')" 2>/dev/null || \
  source "$(grep -sm1 "^$f " "$0.exe.runfiles_manifest" | cut -f2- -d' ')" 2>/dev/null || \
  { echo>&2 "ERROR: cannot find $f"; exit 1; }; f=; set -e
# --- end runfiles.bash initialization v3 ---

BAZEL_BIN="$(rlocation bazel_linux_x86_64/file/bazel)"
readonly BAZEL_BIN

if [[ ! -x "${BAZEL_BIN}" ]]; then
    echo "Failed to find the bazel binary." >&2
    exit 1
fi

export PATH="$(dirname "${BAZEL_BIN}"):${PATH}"

cd tests/ts/bazel_repository_test_dir/

bazel test //...
