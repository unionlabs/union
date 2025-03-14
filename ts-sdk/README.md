# Union TypeScript SDK

`@unionlabs/sdk`

## How to build and test

```sh
nix build .#ts-sdk -L
```

## How to publish


First, bump `version` in `package.json`. Then:

```sh
nix run .#publish-ts-sdk -L
```

## How to develop

```sh
nix develop
cd ts-sdk/
npm install
npm run test-watch
```

## How to update abis

```sh
nix build .#hubble-abis -L
```

copy from `result/` to `src/evm/abi/`
