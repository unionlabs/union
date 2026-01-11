# Voyager EVM State Module

This module provides state querying functionality for the Union IBC stack
deployed on an [EVM]-compatible chain. IBC state is read via [`eth_call`] on the
core contract, and queries are performed via [`eth_getLogs`].

## Configuration

- `rpc_url`: _String_. The [Ethereum JSON-RPC] RPC url for this [EVM]-compatible
  chain.
- `ibc_handler_address`: _H160_. The address of the deployed `IBCHandler.sol`
  contract on this chain. Canonical deployments can be found
  [here][deployments].
- `max_query_window`: _Option<u64>_. The maximum query window to use when
  calling the RPC with [`eth_getLogs`]. If not provided, the full chain will be
  queried (i.e. earliest..latest), which is not supported by many RPC providers.
- `max_cache_size`: _u32_. The cache size to use for RPC-level caching via
  alloy's [`CacheLayer`]. Defaults to 0 if not provided.

### Module Info

This module only provides state for the `ibc-union` IBC specification, and the
[EVM IBC Interface].

[deployments]: https://docs.union.build/protocol/deployments#ibc-solidity
[ethereum json-rpc]: https://ethereum.org/developers/docs/apis/json-rpc
[evm]: https://ethereum.org/developers/docs/evm
[evm ibc interface]: https://docs.union.build/connect/implementations/#evm
[`cachelayer`]: https://docs.rs/alloy/latest/alloy/providers/layers/struct.CacheLayer.html
[`eth_call`]: https://ethereum.org/developers/docs/apis/json-rpc/#eth_call
[`eth_getlogs`]: https://ethereum.org/developers/docs/apis/json-rpc/#eth_getLogs
