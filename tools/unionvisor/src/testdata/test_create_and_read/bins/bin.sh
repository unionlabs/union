#!/usr/bin/env sh
set -e

mkdir -p $4
printf %s '{"name": "upgrade1", "height": 123}' > $4/upgrade-info.json
