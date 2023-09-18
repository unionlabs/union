# Cosmos SDK v0.47.5 Release Notes

💬 [**Release Discussion**](https://github.com/orgs/cosmos/discussions/categories/announcements)

## 🚀 Highlights

Get ready for v0.50.0 and start integrating with the next [Cosmos SDK](https://github.com/cosmos/cosmos-sdk/releases/tag/v0.50.0-rc.0) release.

For this 5th patch release of the `v0.47.x` line, some of the notable changes include:

* A new command for importing private keys encoded in hex. This complements the existing `import` command that supports mnemonic and key files.
  Use `<appd> keys import <name> <hex>` to import a private key encoded in hex.
* A new command, `rpc.QueryEventForTxCmd` for querying a transaction by its hash and blocking until the transaction is included in a block. It is useful as an alternative to the legacy `--sync block`.

Check out the [changelog](https://github.com/cosmos/cosmos-sdk/blob/v0.47.5/CHANGELOG.md) for an exhaustive list of changes or [compare changes](https://github.com/cosmos/cosmos-sdk/compare/release/v0.47.4...v0.47.5) from last release.

Refer to the [upgrading guide](https://github.com/cosmos/cosmos-sdk/blob/release/v0.47.x/UPGRADING.md) when migrating from `v0.46.x` to `v0.47.0`.
