# Voyager CosmWasm Proof Module

This module provides state proof querying functionality for the Union IBC stack deployed on [CosmWasm]. State proofs are read through [`abci_query`], under the storage key defined [here][storekeys].

## Configuration

- `rpc_url`: *String*. The [CometBFT] RPC url for this [CosmosSDK] based chain. The node that this points to must support historical state proofs (this is usually enabled by default for all CosmosSDK nodes, however some chains have optimization settings for higher performance that may disable historical state commitments).

- `ibc_host_contract_address`: *Bech32<H256>*. The address of the deployed `ibc-union` contract on this chain. The contract will be checked to see if it exists at startup. Canonical deployments can be found [here][deployments].

### Module Info

This module only provides proofs for the `ibc-union` IBC specification.

[cometbft]: https://cometbft.com
[cosmossdk]: https://docs.cosmos.network/sdk
[cosmwasm]: https://cosmwasm.com
[deployments]: https://docs.union.build/protocol/deployments#ibc-cosmwasm
[storekeys]: https://github.com/CosmWasm/wasmd/blob/v0.61.6/x/wasm/types/keys.go#L62
[`abci_query`]: https://docs.cometbft.com/v0.38/rpc/#/ABCI/abci_query
