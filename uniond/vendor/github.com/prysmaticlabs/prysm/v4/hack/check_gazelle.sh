#!/bin/bash

# Continuous Integration script to check that BUILD.bazel files are as expected
# when generated from gazelle.

# Duplicate redirect 5 to stdout so that it can be captured, but still printed
# nicely.
exec 5>&1

# Run gazelle while piping a copy of the output to stdout via 5.
changes=$(bazel --batch --bazelrc=.buildkite-bazelrc run //:gazelle -- fix --mode=diff | tee >(cat - >&5))

# If the captured stdout is not empty then Gazelle has diffs.
if [ -z "$changes" ]
then
  echo "OK: Gazelle does not need to be run"
  exit 0
else
  echo "FAIL: Gazelle needs to be run"
  echo "Please run bazel run //:gazelle -- fix"
  exit 1
fi
