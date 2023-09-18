#!/usr/bin/env bash

set -euo pipefail

if [ -z ${1+x} ]; then
  echo "USAGE:
    ./scripts/go-update-dep-all.sh <go-mod-dependency>
  This command updates a dependency in all of the go.mod files which import it.
  It should be called with a single argument which is the go module path of the dependency,
  with an optional version specified by @."
  exit
fi

dependency=$1
# in case the user explicitly specified a dependency version with @, we separate
# the dependency module name into dependency_mod
IFS='@' read -ra dependency_mod <<< "$dependency"
dependency_mod=${dependency_mod[0]}

for modfile in $(find . -name go.mod); do
  if grep $dependency_mod $modfile &> /dev/null; then
    echo "Updating $modfile"
    DIR=$(dirname $modfile)
    # we want to skip the go.mod of the package we're updating
    if [[ "$dependency_mod" == *"$(basename $DIR)"  ]]; then
        echo "Skipping $DIR"
        continue
    fi
     (cd $DIR; go get -u $dependency)
  fi
done
