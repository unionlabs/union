#!/bin/bash
# Copyright 2019 Google LLC
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#      http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License..

# TODO(deklerk): Add integration tests when it's secure to do so. b/64723143

# Fail on any error
set -eo pipefail

# Display commands being run
set -x

# cd to project dir on Kokoro instance
cd github/google-cloud-go

go version

export GOCLOUD_HOME=$KOKORO_ARTIFACTS_DIR/google-cloud-go/
export PATH="$GOPATH/bin:$PATH"
export GO111MODULE=on
export GOPROXY=https://proxy.golang.org

# Move code into artifacts dir
mkdir -p $GOCLOUD_HOME
git clone . $GOCLOUD_HOME
cd $GOCLOUD_HOME

try3() { eval "$*" || eval "$*" || eval "$*"; }

# All packages, including +build tools, are fetched.
try3 go mod download

set +e # Run all tests, don't stop after the first failure.
exit_code=0

# Run tests in the current directory and tee output to log file,
# to be pushed to GCS as artifact.
runPresubmitTests() {
  if [[ $PWD == *"/internal/"* ]] ||
    [[ $PWD == *"/third_party/"* ]]; then
    # internal tools only expected to work with latest go version
    return
  fi

  if [ -z ${RUN_INTEGRATION_TESTS} ]; then
    GOWORK=off go test -race -v -timeout 15m -short ./... 2>&1 |
      tee sponge_log.log
  else
    GOWORK=off go test -race -v -timeout 45m ./... 2>&1 |
      tee sponge_log.log
  fi

  # Run integration tests against an emulator.
  if [ -f "emulator_test.sh" ]; then
    ./emulator_test.sh
  fi
  # Takes the kokoro output log (raw stdout) and creates a machine-parseable
  # xUnit XML file.
  cat sponge_log.log |
    go-junit-report -set-exit-code >sponge_log.xml
  # Add the exit codes together so we exit non-zero if any module fails.
  exit_code=$(($exit_code + $?))
  if [[ $PWD != *"/internal/"* ]]; then
    GOWORK=off go build ./...
  fi
  exit_code=$(($exit_code + $?))
}

SIGNIFICANT_CHANGES=$(git --no-pager diff --name-only origin/main...$KOKORO_GIT_COMMIT_google_cloud_go |
  grep -Ev '(\.md$|^\.github|\.json$|\.yaml$)' | xargs dirname | sort -u || true)

if [ -z $SIGNIFICANT_CHANGES ]; then
  echo "No changes detected, skipping tests"
  exit 0
fi

# CHANGED_DIRS is the list of significant top-level directories that changed,
# but weren't deleted by the current PR. CHANGED_DIRS will be empty when run on main.
CHANGED_DIRS=$(echo "$SIGNIFICANT_CHANGES" | tr ' ' '\n' | cut -d/ -f1 | sort -u |
  tr '\n' ' ' | xargs ls -d 2>/dev/null || true)

echo "Running tests only in changed submodules: $CHANGED_DIRS"
for d in $CHANGED_DIRS; do
  # Check if "." is in the list of changed directories, which means the root
  if [ "$d" = "." ]; then
    pushd $(dirname $d)
    runPresubmitTests
    popd
    continue
  fi
  for i in $(find "$d" -name go.mod); do
    pushd $(dirname $i)
    runPresubmitTests
    popd
  done
done

exit $exit_code
