#!/bin/bash

# This script serves as a wrapper around bazel to limit the scope of environment variables that
# may change the action output. Using this script should result in a higher cache hit ratio for
# cached actions with a more heremtic build.

env -i \
 PATH=/usr/bin:/bin \
 HOME="$HOME" \
 GOOGLE_APPLICATION_CREDENTIALS="$GOOGLE_APPLICATION_CREDENTIALS" \
 bazel "$@"
