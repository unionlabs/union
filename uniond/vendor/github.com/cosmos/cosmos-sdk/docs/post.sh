#!/usr/bin/env bash

find docs/modules ! -name '_category_.json' -type f -exec rm -rf {} +
rm -rf docs/tooling/01-cosmovisor.md
rm -rf docs/tooling/02-depinject.md
rm -rf docs/run-node/04-rosetta.md
rm -rf docs/architecture
rm -rf docs/spec
rm -rf docs/migrations/02-upgrading.md
rm -rf versioned_docs versioned_sidebars versions.json
