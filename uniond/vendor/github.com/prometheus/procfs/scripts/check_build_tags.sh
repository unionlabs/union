#!/usr/bin/env bash

fail=0
while read -r f ; do 
  if ! grep -q '+build linux' "$f" ; then
    echo "missing linux build tag: $f"
    fail=1
  fi
done < <(find sysfs -name '*.go')

exit "${fail}"

