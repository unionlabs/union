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
# limitations under the License.

##
# continuous.sh
# Runs CI checks for entire repository.
#
# Jobs types
#
# Continuous: Runs root tests & tests in submodules changed by a PR. Triggered by PR merges.
# Nightly: Runs root tests & tests in all modules. Triggered nightly.
# Nightly/$MODULE: Runs tests in a specified module. Triggered nightly.
##

export GOOGLE_APPLICATION_CREDENTIALS=$KOKORO_KEYSTORE_DIR/72523_go_integration_service_account
# Removing the GCLOUD_TESTS_GOLANG_PROJECT_ID setting may make some integration
# tests (like profiler's) silently skipped, so make sure you know what you are
# doing when changing / removing the next line.

export GCLOUD_TESTS_GOLANG_PROJECT_ID=dulcet-port-762
export GCLOUD_TESTS_GOLANG_SECONDARY_BIGTABLE_PROJECT_ID=gcloud-golang-firestore-tests
export GCLOUD_TESTS_GOLANG_KEY=$GOOGLE_APPLICATION_CREDENTIALS
export GCLOUD_TESTS_GOLANG_DATASTORE_DATABASES=database-01
export GCLOUD_TESTS_GOLANG_FIRESTORE_PROJECT_ID=gcloud-golang-firestore-tests
export GCLOUD_TESTS_GOLANG_FIRESTORE_KEY=$KOKORO_KEYSTORE_DIR/72523_go_firestore_integration_service_account
export GCLOUD_TESTS_GOLANG_FIRESTORE_DATABASES=database-02
export GCLOUD_TESTS_API_KEY=$(cat $KOKORO_KEYSTORE_DIR/72523_go_gcloud_tests_api_key)
export GCLOUD_TESTS_GOLANG_KEYRING=projects/dulcet-port-762/locations/us/keyRings/go-integration-test
export GCLOUD_TESTS_GOLANG_PROFILER_ZONE="us-west1-b"
export GCLOUD_TESTS_IMPERSONATE_READER_KEY="${KOKORO_GFILE_DIR}/secret_manager/go-cloud-integration-impersonate-reader-service-account"
export GCLOUD_TESTS_IMPERSONATE_READER_EMAIL="impersonate-reader@${GCLOUD_TESTS_GOLANG_PROJECT_ID}.iam.gserviceaccount.com"
export GCLOUD_TESTS_IMPERSONATE_WRITER_EMAIL="impersonate-writer@${GCLOUD_TESTS_GOLANG_PROJECT_ID}.iam.gserviceaccount.com"
export GCLOUD_TESTS_GOLANG_PROJECT_NUMBER=$(cat ${KOKORO_GFILE_DIR}/secret_manager/go-cloud-integration-project-number)
export GCLOUD_TESTS_GOLANG_SERVICE_ACCOUNT_CLIENT_ID=$(cat ${KOKORO_GFILE_DIR}/secret_manager/go-cloud-integration-byoid-client-id)
export GCLOUD_TESTS_GOLANG_AWS_ACCOUNT_ID=$(cat ${KOKORO_GFILE_DIR}/secret_manager/go-cloud-integration-byoid-aws-acc-id)
export GCLOUD_TESTS_GOLANG_AWS_ROLE_NAME=$(cat ${KOKORO_GFILE_DIR}/secret_manager/go-cloud-integration-byoid-aws-role-name)
export GCLOUD_TESTS_GOLANG_AWS_ROLE_ID="arn:aws:iam::$GCLOUD_TESTS_GOLANG_AWS_ACCOUNT_ID:role/$GCLOUD_TESTS_GOLANG_AWS_ROLE_NAME"
export GCLOUD_TESTS_GOLANG_AUDIENCE_OIDC=$(cat ${KOKORO_GFILE_DIR}/secret_manager/go-cloud-integration-byoid-aud-oidc)
export GCLOUD_TESTS_GOLANG_AUDIENCE_AWS=$(cat ${KOKORO_GFILE_DIR}/secret_manager/go-cloud-integration-byoid-aud-aws)
export GOOGLE_EXTERNAL_ACCOUNT_ALLOW_EXECUTABLES="1"
export GOOGLE_API_GO_EXPERIMENTAL_ENABLE_NEW_AUTH_LIB="true"

# Bigtable integration tests expect an existing instance and cluster
#  ❯ cbt createinstance gc-bt-it-instance gc-bt-it-instance \
#    gc-bt-it-cluster us-west1-b 1 SSD
export GCLOUD_TESTS_BIGTABLE_KEYRING=projects/dulcet-port-762/locations/us-central1/keyRings/go-integration-test-regional
export GCLOUD_TESTS_BIGTABLE_CLUSTER="gc-bt-it-cluster"
export GCLOUD_TESTS_BIGTABLE_PRI_PROJ_SEC_CLUSTER="gc-bt-it-cluster-02"
export GCLOUD_TESTS_BIGTABLE_INSTANCE="gc-bt-it-instance"

# TODO: Remove this env after OMG/43748 is fixed
# Spanner integration tests for backup/restore is flaky https://github.com/googleapis/google-cloud-go/issues/5037
# to fix the flaky test Spanner need to run on us-west1 region.
export GCLOUD_TESTS_GOLANG_SPANNER_INSTANCE_CONFIG="regional-us-west1"

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

# runDirectoryTests runs all tests in the current directory.
# If a PATH argument is specified, it runs `go test [PATH]`.
runDirectoryTests() {
  if { [[ $PWD == *"/internal/"* ]] ||
    [[ $PWD == *"/third_party/"* ]]; } &&
    [[ $KOKORO_JOB_NAME == *"earliest"* ]]; then
    # internal tools only expected to work with latest go version
    return
  fi
  GOWORK=off go test -race -v -timeout 45m "${1:-./...}" 2>&1 |
    tee sponge_log.log
  # Takes the kokoro output log (raw stdout) and creates a machine-parseable
  # xUnit XML file.
  cat sponge_log.log |
    go-junit-report -set-exit-code >sponge_log.xml
  # Add the exit codes together so we exit non-zero if any module fails.
  exit_code=$(($exit_code + $?))
}

# runEmulatorTests runs emulator tests in the current directory.
runEmulatorTests() {
  if [ -f "emulator_test.sh" ]; then
    ./emulator_test.sh
    # Takes the kokoro output log (raw stdout) and creates a machine-parseable
    # xUnit XML file.
    cat sponge_log.log |
      go-junit-report -set-exit-code >sponge_log.xml
    # Add the exit codes together so we exit non-zero if any module fails.
    exit_code=$(($exit_code + $?))
  fi
}

# testAllModules runs all modules' tests, including emulator tests.
testAllModules() {
  echo "Testing all modules"
  for i in $(find . -name go.mod); do
    pushd "$(dirname "$i")" >/dev/null
    runDirectoryTests
    # Run integration tests against an emulator.
    runEmulatorTests
    popd >/dev/null
  done
}

# testChangedModules runs tests in changed modules only.
testChangedModules() {
  for d in $CHANGED_DIRS; do
    goDirectories="$(find "$d" -name "*.go" -printf "%h\n" | sort -u)"
    if [[ -n "$goDirectories" ]]; then
      for gd in $goDirectories; do
        pushd "$gd" >/dev/null
        runDirectoryTests .
        popd >/dev/null
      done
    fi
  done
}

set +e # Run all tests, don't stop after the first failure.
exit_code=0

if [[ $KOKORO_JOB_NAME == *"continuous"* ]]; then
  # Continuous jobs only run root tests & tests in submodules changed by the PR.
  SIGNIFICANT_CHANGES=$(git --no-pager diff --name-only $KOKORO_GIT_COMMIT^..$KOKORO_GIT_COMMIT | grep -Ev '(\.md$|^\.github|\.json$|\.yaml$)' || true)

  if [ -z $SIGNIFICANT_CHANGES ]; then
    echo "No changes detected, skipping tests"
    exit 0
  fi

  # CHANGED_DIRS is the list of significant top-level directories that changed,
  # but weren't deleted by the current PR. CHANGED_DIRS will be empty when run on main.
  CHANGED_DIRS=$(echo "$SIGNIFICANT_CHANGES" | tr ' ' '\n' | grep "/" | cut -d/ -f1 | sort -u | tr '\n' ' ' | xargs ls -d 2>/dev/null || true)
  if [[ -n $TARGET_MODULE ]]; then
    pushd $TARGET_MODULE >/dev/null
    runDirectoryTests
    popd >/dev/null
  elif [[ -z $SIGNIFICANT_CHANGES ]] || echo "$SIGNIFICANT_CHANGES" | tr ' ' '\n' | grep "^go.mod$" || [[ $CHANGED_DIRS =~ "internal" ]]; then
    # If PR changes affect all submodules, then run all tests.
    testAllModules
  else
    runDirectoryTests . # Always run base tests.
    echo "Running tests only in changed submodules: $CHANGED_DIRS"
    testChangedModules
  fi
elif [[ $KOKORO_JOB_NAME == *"nightly"* ]]; then
  # Expected job name format: ".../nightly/[OPTIONAL_MODULE_NAME]/[OPTIONAL_JOB_NAMES...]"
  ARR=(${KOKORO_JOB_NAME//// }) # Splits job name by "/" where ARR[0] is expected to be "nightly".
  SUBMODULE_NAME=${ARR[5]}      # Gets the token after "nightly/".
  if [[ -n $SUBMODULE_NAME ]] && [[ -d "./$SUBMODULE_NAME" ]]; then
    # Only run tests in the submodule designated in the Kokoro job name.
    # Expected format example: ...google-cloud-go/nightly/logging.
    runDirectoryTests . # Always run base tests
    echo "Running tests in one submodule: $SUBMODULE_NAME"
    pushd $SUBMODULE_NAME >/dev/null
    runDirectoryTests
    popd >/dev/null
  else
    # Run all tests if it is a regular nightly job.
    testAllModules
  fi
else
  testAllModules
fi

if [[ $KOKORO_BUILD_ARTIFACTS_SUBDIR = *"continuous"* ]] || [[ $KOKORO_BUILD_ARTIFACTS_SUBDIR = *"nightly"* ]]; then
  chmod +x $KOKORO_GFILE_DIR/linux_amd64/flakybot
  $KOKORO_GFILE_DIR/linux_amd64/flakybot -logs_dir=$GOCLOUD_HOME \
    -repo=googleapis/google-cloud-go \
    -commit_hash=$KOKORO_GITHUB_COMMIT_URL_google_cloud_go
fi

exit $exit_code
