#!/usr/bin/env bash

readarray -t mod_files < <(find . -type f -name go.mod)

echo "Checking files ${mod_files[@]}"

matches=$(awk '$1 == "go" {print $2}' "${mod_files[@]}" | sort -u | wc -l)

if [[ "${matches}" -ne 1 ]]; then
  echo 'Not all go.mod files have matching go versions'
  exit 1
fi
