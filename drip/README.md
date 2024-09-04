# Drip

Faucet for Cosmos chains: [app.union.build/faucet]

[app.union.build/faucet]: https://app.union.build/faucet

## Example usage

Commands are ran from repo root

Tab 1, Union Devnet:
```sh
nix run .#devnet-union -L
```

Tab 2, Drip:
```sh
nix run .#drip -- -c ./drip/config.json
```

Tab 3, Request:
```sh
cat ./drip/example-request.json | http POST localhost:8000
```
