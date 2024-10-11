#!/bin/bash

# Copyright 2022 Google LLC.
# Use of this source code is governed by a BSD-style
# license that can be found in the LICENSE file.

# Fail on any error
set -eo pipefail

export GITHUB_ACCESS_TOKEN=$(cat $KOKORO_KEYSTORE_DIR/73713_yoshi-automation-github-key)

# Display commands being run
set -x

# cd to project dir on Kokoro instance
cd github/google-api-go-client
export DISCOVERY_DIR=$(pwd)
git config --global --add safe.directory $PWD

cd internal/kokoro/discogen
go run google.golang.org/api/internal/kokoro/discogen
