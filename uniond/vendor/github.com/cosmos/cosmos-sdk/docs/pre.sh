#!/usr/bin/env bash

## Create modules pages
for D in ../x/*; do
  if [ -d "${D}" ]; then
    MODDOC=docs/modules/$(echo $D | awk -F/ '{print $NF}')
    rm -rf $MODDOC
    mkdir -p $MODDOC && cp -r $D/README.md "$_"
  fi
done

## Vesting is a submodule of auth, but we still want to display it in docs
## TODO to be removed in https://github.com/cosmos/cosmos-sdk/issues/9958
cp ../x/auth/vesting/README.md ./docs/modules/auth/1-vesting.md
cp ../x/auth/tx/README.md ./docs/modules/auth/2-tx.md

## Add modules page list
cat ../x/README.md | sed 's/\.\.\/docs\/building-modules\/README\.md/\/building-modules\/intro\.html/g' > ./docs/modules/README.md

## Add cosmovisor documentation
cp ../tools/cosmovisor/README.md ./docs/tooling/01-cosmovisor.md

## Add depinject documentation
cp ../depinject/README.md ./docs/tooling/02-depinject.md

## Add rosetta documentation
cp ../tools/rosetta/README.md ./docs/run-node/04-rosetta.md

## Add architecture documentation
cp -r ./architecture ./docs

## Add spec documentation
cp -r ./spec ./docs

## Add SDK migration documentation
cp -r ../UPGRADING.md ./docs/migrations/02-upgrading.md
