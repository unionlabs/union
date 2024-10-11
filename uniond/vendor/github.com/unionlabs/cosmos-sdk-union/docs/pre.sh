#!/usr/bin/env bash

## Create modules pages
for D in ../x/*; do
  if [ -d "${D}" ]; then
    DIR_NAME=$(basename "$D")
    
    # Skip specific directories
    if [[ "$DIR_NAME" != "counter" ]]; then
      MODULE_DIRECTORY=docs/build/modules/$DIR_NAME
      rm -rf "$MODULE_DIRECTORY"
      mkdir -p "$MODULE_DIRECTORY" 
      if [ -f "$D"/README.md ]; then
        cp -r "$D"/README.md "$MODULE_DIRECTORY"
      fi
    fi
  fi
done

## Vesting is a submodule of auth, but we still want to display it in docs
## TODO to be removed in https://github.com/cosmos/cosmos-sdk/issues/9958
cp -r ../x/auth/vesting/README.md ./build/modules/auth/1-vesting.md
cp -r ../x/auth/tx/README.md ./build/modules/auth/2-tx.md

## Add modules page list
cat ../x/README.md | sed 's/\.\.\/\/build\/building-modules\/README\.md/\/building-modules\/intro\.html/g' > ./build/modules/README.md

## Add tooling documentation
wget -O ./build/tooling/01-cosmovisor.md https://raw.githubusercontent.com/cosmos/cosmos-sdk/main/tools/cosmovisor/README.md
wget -O ./build/tooling/02-confix.md https://raw.githubusercontent.com/cosmos/cosmos-sdk/main/tools/confix/README.md
wget -O ./build/tooling/03-hubl.md https://raw.githubusercontent.com/cosmos/cosmos-sdk/main/tools/hubl/README.md

## Add package documentation
cp ../client/v2/README.md ./learn/advanced/17-autocli.md

wget -O ./build/packages/01-depinject.md https://raw.githubusercontent.com/cosmos/cosmos-sdk/main/depinject/README.md
wget -O ./build/packages/02-collections.md https://raw.githubusercontent.com/cosmos/cosmos-sdk/main/tools/collections/README.md
wget -O ./build/packages/03-orm.md https://raw.githubusercontent.com/cosmos/cosmos-sdk/main/tools/orm/README.md

## Add architecture documentation
cp -r ./architecture ./build

## Add spec documentation
cp -r ./spec ./build

## Add rfc documentation
cp -r ./rfc ./build

## Add SDK migration documentation
cp -r ../UPGRADING.md ./build/migrations/02-upgrading.md
