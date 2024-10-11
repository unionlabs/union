#!/bin/bash
# Copyright 2021 Google LLC
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

# A suite of tests offered by https://github.com/googleapis/env-tests-logging
# That allows deploying and testing features in live GCP services.
# Currently only configured to test logging & error reporting

set -eo pipefail

# Test prechecks
if [[ -z "${ENVIRONMENT:-}" ]]; then
  echo "ENVIRONMENT not set. Exiting"
  exit 1
fi

if [[ -z "${PROJECT_ROOT:-}"  ]]; then
    PROJECT_ROOT="github/google-cloud-go"
fi

# add kokoro labels for testgrid filtering
export PRODUCT_AREA_LABEL=observability
export PRODUCT_LABEL=logging
export LANGUAGE_LABEL=go

# Add the test module as a submodule to the repo.
cd "${KOKORO_ARTIFACTS_DIR}/github/google-cloud-go/internal/"
git submodule add https://github.com/googleapis/env-tests-logging
cd "env-tests-logging/"
export ENV_TEST_PY_VERSION=3.9
echo "using python version: $ENV_TEST_PY_VERSION"

# run tests from git tag golang-envtest-pin when available
TAG_ID="golang-envtest-pin"
git fetch --tags
if [ $(git tag -l "$TAG_ID")  ]; then
  git -c advice.detachedHead=false checkout $TAG_ID
else
  echo "WARNING: tag $TAG_ID not found in repo"
fi
echo "running env-tests-logging at commit: $(git rev-parse HEAD)"

# Disable buffering, so that the logs stream through.
export PYTHONUNBUFFERED=1

# Debug: show build environment
env | grep KOKORO

# Set up service account credentials
export GOOGLE_APPLICATION_CREDENTIALS=$KOKORO_KEYSTORE_DIR/72523_go_integration_service_account
gcloud auth activate-service-account --key-file=$GOOGLE_APPLICATION_CREDENTIALS

set -x
export PROJECT_ID="dulcet-port-762"
gcloud config set project $PROJECT_ID
gcloud config set compute/zone us-central1-b

# Authenticate docker
gcloud auth configure-docker -q

# create a unique id for this run
UUID=$(python3  -c 'import uuid; print(uuid.uuid1())' | head -c 7)
export ENVCTL_ID=ci-$UUID
echo $ENVCTL_ID

# If App Engine, install app-engine-go component
if [[ $ENVIRONMENT == *"appengine"* ]]; then
  apt-get install google-cloud-sdk-app-engine-go -y | cat
fi

# If Kubernetes, install kubectl component
if [[ $ENVIRONMENT == *"kubernetes"* ]]; then
  curl -LO "https://dl.k8s.io/$(curl -L -s https://dl.k8s.io/release/stable.txt)/bin/linux/amd64/kubectl"
  chmod +x kubectl
  mkdir -p ~/.local/bin
  mv ./kubectl ~/.local/bin
  export PATH=$PATH:~/.local/bin/
  # install auth plugin
  apt-get install google-cloud-sdk-gke-gcloud-auth-plugin
  export USE_GKE_GCLOUD_AUTH_PLUGIN=True
fi

# If Functions, use python3.8, since that's what's in go116 container
if [[ $ENVIRONMENT == *"functions"* ]]; then
  export ENV_TEST_PY_VERSION=3.8
  python3 -m pip install nox
fi

# Run the environment test for the specified GCP service
set +e
python3 -m nox --session "tests(language='go', platform='$ENVIRONMENT')"
TEST_STATUS_CODE=$?

# destroy resources
echo "cleaning up..."
./envctl/envctl go $ENVIRONMENT destroy

# exit with proper status code
exit $TEST_STATUS_CODE
