#!/usr/bin/env bash

find build/modules ! -name '_category_.json' -type f -exec rm -rf {} +
rm -rf build/tooling/01-cosmovisor.md
rm -rf build/tooling/02-confix.md
rm -rf build/tooling/03-hubl.md
rm -rf build/packages/01-depinject.md
rm -rf build/packages/02-collections.md
rm -rf build/packages/03-orm.md
rm -rf learn/advaced-concepts/17-autocli.md
rm -rf build/architecture
rm -rf build/spec
rm -rf build/rfc
rm -rf learn/advanced/17-autocli.md
rm -rf build/migrations/02-upgrading.md
