#!/usr/bin/env bash

BASE_SHA="$1"
HEAD_SHA="$2"

if [ -z "$HEAD_SHA" ];then
    echo "Usage: $0 <base-sha> <head-sha>"
    exit 1
fi

git diff --name-only "${BASE_SHA}..${HEAD_SHA}" -- "*.go" \
  | xargs -rn1 dirname \
  | sort -u \
  | xargs echo
