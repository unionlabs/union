#!/usr/bin/env bash

# This script runs all the examples in the playground directory
# if one of the examples fails, it will continue to the next one

# list all the files in the playground directory
for file in playground/*.ts; do
  echo "$file"
  # bun "$file" --private-key "1bdd5c2105f62c51d72c90d9e5ca6854a94337bcbcbb0b959846b85813d69380" || true
done
