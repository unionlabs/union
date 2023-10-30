#!/usr/bin/env bash
set -e

# switch to main branch and record its bench results
git checkout main && \
cargo bench --bench benchmark -- --noplot --save-baseline before

# record current bench results
git checkout -f - && \
cargo bench --bench benchmark -- --noplot --baseline before
