#!/usr/bin/env bash
# Copyright 2024 gRPC authors.
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.
set -eo pipefail

#######################################
# Builds test app Docker images and pushes them to GCR.
# Called from psm_interop_kokoro_lib.sh.
#
# Globals:
#   SRC_DIR: Absolute path to the source repo on Kokoro VM
#   SERVER_IMAGE_NAME: Test server Docker image name
#   CLIENT_IMAGE_NAME: Test client Docker image name
#   GIT_COMMIT: SHA-1 of git commit being built
#   DOCKER_REGISTRY: Docker registry to push to
# Outputs:
#   Writes the output of docker image build stdout, stderr
#######################################
psm::lang::build_docker_images() {
  local client_dockerfile="interop/xds/client/Dockerfile"
  local server_dockerfile="interop/xds/server/Dockerfile"
  psm::build::docker_images_generic "${client_dockerfile}" "${server_dockerfile}"
}
