#!/bin/bash

# This script is used to tag docker images with a specific commit or tag the
# docker images as ":stable" with the --stable flag.

# List of docker tags to update with git versioned tag.
DOCKER_IMAGES=(
  # Beacon chain images
  "gcr.io/prysmaticlabs/prysm/beacon-chain"
  "index.docker.io/prysmaticlabs/prysm-beacon-chain"
  # Validator images
  "gcr.io/prysmaticlabs/prysm/validator"
  "index.docker.io/prysmaticlabs/prysm-validator"
  # Slasher images
  "gcr.io/prysmaticlabs/prysm/slasher"
  "index.docker.io/prysmaticlabs/prysm-slasher"
)


# Check that the current commit has an associated git tag.
TAG=$(git describe --tags HEAD)
TAG_COMMIT=$(git rev-list -n 1 "$TAG")
CURRENT_COMMIT=$(git rev-parse HEAD)

if [ "$TAG_COMMIT" != "$CURRENT_COMMIT" ]
then
  echo "Current commit does not have an associated tag."
  exit 1
fi

TAG_AS_STABLE=0

for arg in "$@"
do
    case $arg in
        -s|--stable)
        TAG_AS_STABLE=1
        shift # Remove --stable from processing
        ;;
        *)
        OTHER_ARGUMENTS+=("$1")
        shift # Remove generic argument from processing
        ;;
    esac
done

if [ "$TAG_AS_STABLE" = "1" ]
then
  TAG="stable"
fi

HEAD=HEAD-$(git rev-parse --short=6 HEAD)

for image in "${DOCKER_IMAGES[@]}"
do
  SRC="$image:$HEAD"
  DST="$image:$TAG"
  echo "Pulling $SRC"
  docker pull "$SRC"
  echo "Tagging $SRC as $DST"
  docker tag "$SRC" "$DST"
  echo "Pushing $DST"
  docker push "$DST"
done

exit 0
