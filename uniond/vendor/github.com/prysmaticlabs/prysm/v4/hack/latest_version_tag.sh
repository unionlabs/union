#!/usr/bin/env bash
set -e

# Prints the latest git version tag, like "v2.12.8"
git tag -l 'v*' --sort=creatordate |
    perl -nle 'if (/^v\d+\.\d+\.\d+$/) { print $_ }' |
    tail -n1

