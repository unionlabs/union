# Scripts

Generally we should avoid shell scripting and write tests purely in Golang.
However, some libraries are not Goroutine-safe (e.g. app simulations cannot be run safely in parallel),
and OS-native threading may be more efficient for many parallel simulations, so we use shell scripts here.

## Validate Gentxs

A custom utility script is available to [validate gentxs](./validate-gentxs.sh). Though we have
`ValidateBasic()` for validating gentx data, it cannot validate signatures. This custom script helps
to validate all the gentxs by collecting them one by one and starting a local network.
It requires the following env settings.

```shell
export DAEMON=gaiad
export CHAIN_ID=cosmoshub-1
export DENOM=uatom
export GH_URL=https://github.com/cosmos/gaia
export BINARY_VERSION=v1.0.0
export GO_VERSION=1.17
export PRELAUNCH_GENESIS_URL=https://raw.githubusercontent.com/cosmos/mainnet/main/cosmoshub-1/genesis-prelaunch.json
export GENTXS_DIR=~/go/src/github.com/cosmos/mainnet/$CHAIN_ID/gentxs
```

Though this script is handy for verifying the gentxs locally, it is advised to use Github Action to validate gentxs.
An example can be found here:
https://github.com/regen-network/mainnet/blob/0bcd387671b9574e893289e39c08a1643cac7d62/.github/workflows/validate-gentx.yml
