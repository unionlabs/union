# Drip

Faucet for Cosmos chains: [Faucet][app.union.build/faucet]. Supports multiple chains and multiple denoms per chains.

## Example usage

Commands are run from repo root

Tab 1, Union Devnet:

```sh
nix run .#devnet-union -L

```

Tab 2, Stargaze Devnet (optional, multi-chain demo):

```sh
nix run .#devnet-stargaze -L
```

Tab 3, Drip:

```sh
nix run .#drip -- -c ./drip/config.json
```

Tab 4, Request:

```sh
cat ./drip/example-requests/union-devnet.json | http POST localhost:8000
cat ./drip/example-requests/stargaze-devnet.json | http POST localhost:8000
```

[app.union.build/faucet]: https://app.union.build/faucet
