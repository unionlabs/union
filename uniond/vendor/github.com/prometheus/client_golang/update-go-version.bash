#!/bin/env bash

set -e

get_latest_versions() {
  curl -s https://go.dev/VERSION?m=text | sed -E -n 's/go([0-9]+\.[0-9]+|\.[0-9]+).*/\1/p'
}

current_version=$(cat supported_go_versions.txt | head -n 1)
latest_version=$(get_latest_versions)

# Check for new version of Go, and generate go collector test files
# Add new Go version to supported_go_versions.txt, and remove the oldest version
if [[ ! $current_version =~ $latest_version ]]; then
  echo "New Go version available: $latest_version"
  echo "Updating supported_go_versions.txt and generating Go Collector test files"
  sed -i "1i $latest_version" supported_go_versions.txt
  sed -i '$d' supported_go_versions.txt
  make generate-go-collector-test-files
else
  echo "No new Go version detected. Current Go version is: $current_version"
fi

