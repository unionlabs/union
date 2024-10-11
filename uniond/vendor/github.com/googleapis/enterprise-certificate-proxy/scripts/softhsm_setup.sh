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

SOFTHSM2_MODULE="/usr/lib/x86_64-linux-gnu/softhsm/libsofthsm2.so"
TOKEN_NAME="Demo Token"
OBJECT_LABEL="Demo Object"
PIN="0000"

install_dependencies() {
  # Install PKCS #11 related dependencies.
  # 1. softhsm2 is a software based HSM that implements the PKCS #11 spec.
  # 2. libp11-kit-dev contains a shared library at we will use to interact with
  # PKCS #11 device module, as well as pkcs11-tool which will be used for
  # interacting with the PKCS #11 module.
  # 3. gnutls-bin contains p11-tool which we will use to create PKCS #11 URIs.
  sudo apt install softhsm2 libp11-kit-dev gnutls-bin opensc
}

setup_pkcs11_module() {
  # Make softhsm2 discoverable by PKCS #11 tools.
  sudo mkdir -p /etc/pkcs11/modules && echo "module: /usr/lib/softhsm/libsofthsm2.so" | sudo tee -a /etc/pkcs11/modules/softhsm.module

  # Create folder for storing PKCS #11 objects
  mkdir -p $HOME/.config/softhsm2/tokens

  cat <<EOF > $HOME/.config/softhsm2/softhsm2.conf
directories.tokendir = $HOME/.config/softhsm2/tokens/
objectstore.backend = file
log.level = INFO
slots.removable = true
EOF


  pkcs11-tool --init-token --label "$TOKEN_NAME" --module $SOFTHSM2_MODULE --slot 0 --so-pin $PIN
  SLOT=$(pkcs11-tool --list-slots --module $SOFTHSM2_MODULE | grep  -Eo "0x[A-Fa-f0-9]+" | head -n 1)
  pkcs11-tool --module $SOFTHSM2_MODULE --token-label "$TOKEN_NAME" --login --init-pin --pin $PIN --so-pin $PIN


  BUILD_DIR=$(mktemp -d)
  pushd $BUILD_DIR

  openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -sha256 -days 365 -nodes -subj "/C=US/ST=WA/L=Sea/O=My Inc/OU=DevOps/CN=www.example.com/emailAddress=dev@www.example.com"
  openssl x509 -pubkey -noout -in cert.pem > public_key.pem

  openssl x509 -in cert.pem -out cert.der -outform der
  openssl rsa -in key.pem -outform DER -out private_key.der
  openssl rsa -inform pem -in public_key.pem -outform der -out public_key.der -pubin

  pkcs11-tool --module $SOFTHSM2_MODULE --slot $SLOT --write-object cert.der --type cert --label "$OBJECT_LABEL" --login --pin $PIN
  pkcs11-tool --module $SOFTHSM2_MODULE --slot $SLOT --write-object private_key.der --type privkey --label "$OBJECT_LABEL" --login --pin $PIN
  pkcs11-tool --module $SOFTHSM2_MODULE --slot $SLOT --write-object public_key.der --type pubkey --label "$OBJECT_LABEL" --login --pin $PIN

  rm -rf $BUILD_DIR

  popd
}

if [ $# -eq 0 ]; then
    install_dependencies
    setup_pkcs11_module
fi
