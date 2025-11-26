# Voyager CosmWasm State Module

This module provides state querying functionality for the Union IBC stack deployed on [CosmWasm]. IBC state is read via smart contract queries to the core contract, and queries are performed via [`tx_search`].

## Configuration

- `rpc_url`: *String*. The [CometBFT] RPC url for this [CosmosSDK] based chain. The node that this points to must have tx indexing enabled.

- `ibc_host_contract_address`: *Bech32<H256>*. The address of the deployed `ibc-union` contract on this chain. The contract will be checked to see if it exists at startup. Canonical deployments can be found [here][deployments].

### Module Info

This module only provides state for the `ibc-union` IBC specification.

[cometbft]: https://cometbft.com
[cosmossdk]: https://docs.cosmos.network/sdk
[cosmwasm]: https://cosmwasm.com
[deployments]: ../../../../deployments/deployments.json
[`tx_search`]: https://docs.cometbft.com/v0.38/rpc/#/Info/tx_search
