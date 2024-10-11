#!/usr/bin/env bash

set -ex

cd /home/travis

VERSION=3.9.1

wget https://github.com/protocolbuffers/protobuf/releases/download/v$VERSION/protoc-$VERSION-linux-x86_64.zip
unzip protoc-$VERSION-linux-x86_64.zip
rm -rf protoc-$VERSION-linux-x86_64.zip

