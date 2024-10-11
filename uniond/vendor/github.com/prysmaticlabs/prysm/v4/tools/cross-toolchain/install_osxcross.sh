#!/bin/sh
. "$(dirname "$0")"/common_osxcross.sh


# darwin
mkdir -p /usr/osxcross
mkdir -p /tmp/osxcross && cd "/tmp/osxcross"
curl -sLo osxcross.tar.gz "https://codeload.github.com/${OSXCROSS_REPO}/tar.gz/${OSXCROSS_SHA1}"
tar --strip=1 -xzf osxcross.tar.gz
rm -f osxcross.tar.gz

curl -sLo tarballs/${OSX_SDK}.tar.xz "https://prysmaticlabs.com/uploads/${OSX_SDK}.tar.xz"

echo "${OSX_SDK_SUM}"  "tarballs/${OSX_SDK}.tar.xz" | sha256sum -c -
yes "" | SDK_VERSION=12.3 OSX_VERSION_MIN=11.0 OCDEBUG=1 ./build.sh

