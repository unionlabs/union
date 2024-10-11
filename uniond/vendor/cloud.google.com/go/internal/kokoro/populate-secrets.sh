#!/bin/bash
# Copyright 2023 Google LLC.
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

set -eo pipefail

function now { date +"%Y-%m-%d %H:%M:%S" | tr -d '\n'; }
function msg { println "$*" >&2; }
function println { printf '%s\n' "$(now) $*"; }

# Populates requested secrets set in SECRET_MANAGER_KEYS from service account:
# kokoro-trampoline@cloud-devrel-kokoro-resources.iam.gserviceaccount.com
SECRET_LOCATION="${KOKORO_GFILE_DIR}/secret_manager"
msg "Creating folder on disk for secrets: ${SECRET_LOCATION}"
mkdir -p ${SECRET_LOCATION}
for key in $(echo ${SECRET_MANAGER_KEYS} | sed "s/,/ /g"); do
  msg "Retrieving secret ${key}"
  docker run --entrypoint=gcloud \
    --volume=${KOKORO_GFILE_DIR}:${KOKORO_GFILE_DIR} \
    gcr.io/google.com/cloudsdktool/cloud-sdk \
    secrets versions access latest \
    --credential-file-override=${KOKORO_GFILE_DIR}/kokoro-trampoline.service-account.json \
    --project cloud-devrel-kokoro-resources \
    --secret ${key} > \
    "${SECRET_LOCATION}/${key}"
  if [[ $? == 0 ]]; then
    msg "Secret written to ${SECRET_LOCATION}/${key}"
  else
    msg "Error retrieving secret ${key}"
  fi
done
