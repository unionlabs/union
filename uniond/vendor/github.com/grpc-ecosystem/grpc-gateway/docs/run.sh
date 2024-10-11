#! /bin/bash

set -e

JEKYLL_VERSION=4
BUNDLE_DIR="/tmp/grpc-gateway-bundle"

if [ ! -d "${BUNDLE_DIR}" ]; then
  mkdir "${BUNDLE_DIR}"

  # Run this to update the Gemsfile.lock
  docker run --rm \
    --volume="${PWD}:/srv/jekyll" \
    -e "JEKYLL_UID=$(id -u)" \
    -e "JEKYLL_GID=$(id -g)" \
    --volume="/tmp/grpc-gateway-bundle:/usr/local/bundle" \
    -it "jekyll/builder:${JEKYLL_VERSION}" \
    bundle update
fi

if [[ ${JEKYLL_GITHUB_TOKEN} == "" ]]; then
  echo "Please set \$JEKYLL_GITHUB_TOKEN before running"
  exit 1
fi

docker run --rm \
  --volume="${PWD}:/srv/jekyll" \
  -p 35729:35729 -p 4000:4000 \
  -e "JEKYLL_UID=$(id -u)" \
  -e "JEKYLL_GID=$(id -g)" \
  -e "JEKYLL_GITHUB_TOKEN=${JEKYLL_GITHUB_TOKEN}" \
  --volume="/tmp/grpc-gateway-bundle:/usr/local/bundle" \
  -it "jekyll/builder:${JEKYLL_VERSION}" \
  jekyll serve
