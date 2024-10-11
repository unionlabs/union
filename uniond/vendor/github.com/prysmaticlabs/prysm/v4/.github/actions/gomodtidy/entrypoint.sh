#!/bin/sh -l
set -e
export PATH="$PATH:/usr/local/go/bin"

cd "$GITHUB_WORKSPACE"

cp go.mod go.mod.orig
cp go.sum go.sum.orig

go mod tidy -compat=1.17

echo "Checking go.mod and go.sum:"
checks=0
if [ "$(diff -s go.mod.orig go.mod | grep -c 'Files go.mod.orig and go.mod are identical')" = 1 ]; then
  echo "- go.mod is up to date."
  checks=$((checks + 1))
else
  echo "- go.mod is NOT up to date."
fi

if [ "$(diff -s go.sum.orig go.sum | grep -c 'Files go.sum.orig and go.sum are identical')" = 1 ]; then
  echo "- go.sum is up to date."
  checks=$((checks + 1))
else
  echo "- go.sum is NOT up to date."
fi

if [ $checks -eq 2 ]; then
  exit 0
fi

# Notify of any issues.
echo "Run 'go mod tidy' to update."
exit 1
