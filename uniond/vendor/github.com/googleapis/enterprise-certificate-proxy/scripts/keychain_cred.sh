#!/bin/bash

# Copyright 2023 Google LLC.
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     https://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

set -eux

PASSWORD="1234"
WORK_DIR=$(mktemp -d)
KEYCHAIN="BuildTest.keychain"
KEYCHAIN_TEST_BINARY=$(echo "$PWD/$(find . -iname keychain.test)")

pushd "${WORK_DIR}"

openssl req -x509 -newkey rsa:2048 -keyout key.pem -out cert.pem -sha256 -days 5 -nodes -subj "/C=US/ST=WA/L=Kirkland/O=Temp/OU=CI/CN=TestIssuer/emailAddress=dev@example.com" 
openssl pkcs12 -inkey key.pem -in cert.pem -export -out cred.p12 -passin pass:${PASSWORD} -passout pass:${PASSWORD}

security create-keychain -p ${PASSWORD} ${KEYCHAIN}

# Disable password prompt timeout
security set-keychain-setting ${KEYCHAIN}

# Put custom keychain on keychain path
security list-keychains -d user -s ${KEYCHAIN}

security default-keychain -s "${KEYCHAIN}"

security import cred.p12 -P ${PASSWORD} -k ${KEYCHAIN} -A
security unlock-keychain -p ${PASSWORD} ${KEYCHAIN}

# Sign the test binary
codesign -s - "${KEYCHAIN_TEST_BINARY}"

# Grab CD Hash of the keychain test binary
KEYCHAIN_TEST_BINARY_CD_HASH=$(codesign --display --verbose=4 "${KEYCHAIN_TEST_BINARY}" 2>&1 | grep 'CDHash=\(.*\)' | cut -d '=' -f 2)

# Need to specify in ACL that the test binary can access the test toolchain
# This is because the `-A` param to allow all applications access to the private key on the import
# command is apparently not enough...
#
# This method was found by comparing the diff of `$ security dump-keychain -a` before and after always
# allowing the test binary access
security set-key-partition-list -S "cdhash:${KEYCHAIN_TEST_BINARY_CD_HASH}" -k ${PASSWORD} ${KEYCHAIN}

popd
