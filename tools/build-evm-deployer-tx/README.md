# `build-evm-deployer-tx`

Build and sign the raw, pre-EIP-155 transaction for deploying the EVM deployer bytecode.

[`cast`] does not allow for signing a transaction without providing a chain id ([EIP-155]). However, to create a transaction that can be submitted on many chains, the chain id cannot be included.

This tool exists only to fill the gaps in cast's functionality, and is intended to be used alongside `cast`.

## Usage

```sh
SIG_HASH="$(nix run .#build-evm-deployer-tx -- signature-hash)"
# the wallet will need to be threaded to cast here (see `cast wallet address --help` for more information)
SIG="$(cast wallet sign --no-hash "$SIG_HASH")"
nix run .#build-evm-deployer-tx -- raw-tx "$SIG"
```

The generated transaction can then be used with `cast publish`.

## Verification

The following source files are used for deployment:

- `Deployer.sol`: [../../evm/scripts/Deployer.sol](../../evm/scripts/Deployer.sol)
- `LibString.sol`: https://github.com/Vectorized/solady/blob/v0.1.12/src/utils/LibString.sol
- `LibBytes.sol`: https://github.com/Vectorized/solady/blob/v0.1.12/src/utils/LibBytes.sol
- `CREATE3`: https://github.com/Vectorized/solady/blob/v0.1.12/src/utils/CREATE3.sol

The compiled ABI can be found at [./abi.json](./abi.json).

Compiler settings:

- Compiler version v0.8.27
- Optimizer enabled
- Optimizer runs: 1000
- Via IR: true
- EVM Version cancun

An example verified deployment can be found [here](https://dashboard.tenderly.co/contract/421614/0xf596932fc019db09faf466288447275bdbf144e3/code), deployed using [this key](../../networks/genesis/devnet-eth/dev-key1.prv).

## Further Reading

This pre-EIP-155 transaction for multi-chain deployments is inspired by the deployment for [multicall3].

[eip-155]: https://eips.ethereum.org/EIPS/eip-155
[multicall3]: https://github.com/mds1/multicall3#new-deployments
[`cast`]: https://book.getfoundry.sh/cast
