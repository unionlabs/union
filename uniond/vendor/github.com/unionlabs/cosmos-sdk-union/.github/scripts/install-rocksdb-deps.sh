#!/usr/bin/env bash
set -Eeuo pipefail

sudo apt update && sudo apt-get install -y libsnappy-dev zlib1g-dev libbz2-dev liblz4-dev libzstd-dev build-essential
