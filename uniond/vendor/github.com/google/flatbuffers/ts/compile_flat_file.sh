#!/bin/bash
# This is a script used by the typescript flatbuffer bazel rules to compile
# a flatbuffer schema (.fbs file) to typescript and then use esbuild to
# generate a single output.
# Note: This relies on parsing the stdout of flatc to figure out how to
# run esbuild.
# --- begin runfiles.bash initialization v2 ---
# Copy-pasted from the Bazel Bash runfiles library v2.
set -uo pipefail; set +e; f=bazel_tools/tools/bash/runfiles/runfiles.bash
source "${RUNFILES_DIR:-/dev/null}/$f" 2>/dev/null || \
  source "$(grep -sm1 "^$f " "${RUNFILES_MANIFEST_FILE:-/dev/null}" | cut -f2- -d' ')" 2>/dev/null || \
  source "$0.runfiles/$f" 2>/dev/null || \
  source "$(grep -sm1 "^$f " "$0.runfiles_manifest" | cut -f2- -d' ')" 2>/dev/null || \
  source "$(grep -sm1 "^$f " "$0.exe.runfiles_manifest" | cut -f2- -d' ')" 2>/dev/null || \
  { echo>&2 "ERROR: cannot find $f"; exit 1; }; f=; set -e
# --- end runfiles.bash initialization v2 ---
set -eu
runfiles_export_envvars
FLATC=$(rlocation com_github_google_flatbuffers/flatc)
TS_FILE=$(${FLATC}  $@  | grep  "Entry point.*generated" | grep -o "bazel-out.*ts")
export PATH="$(rlocation nodejs_linux_amd64/bin/nodejs/bin):${PATH}"
${ESBUILD_BIN} ${TS_FILE} --format=cjs --bundle --outfile="${OUTPUT_FILE}"  --external:flatbuffers --log-level=warning
