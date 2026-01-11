# Voyager EVM Event Source Plugin

pleaselsdflkjs

This plugin provides event sourcing for EVM-compatible chains. All events are
read through the [`Ethereum JSON-RPC API`][ethrpc], specifically the
[`eth_getLogs`] method. In order to run this plugin, both a state module and a
finality module must be loaded for the same chain. All IBC-related state is read
through the state module, allowing this plugin to take advantage of both the
built-in native caching in Voyager, and also potentially any additional state
fetching optimizations specific to the configured state module. All events
emitted by this plugin are considered to be finalized from the perspective of
this plugin, however finality is not managed internally, but the configured
finality module is relied upon for when to consider state finalized. This means
that this plugin can be used for any EVM-compatible chain, regardless of the
consensus or actual execution environment.

<!-- TODO: Link -->

This plugin complies with the Voyager indexing plugin interface, and as such can
be triggered via the CLI with `voyager index`.

## Config

- `chain_id`: _String_. The expected chain id of the EVM-compatible chain this
  plugin will fetch events from, as a string. `rpc_url` will be checked against
  this on startup (via `eth_chainId`).
- `ibc_handler_address`: _H160_. The address of the Union `IBCHandler` smart
  contract on this chain. All events will be fetched from this contract.
  Canonical deployments can be found [here][deployments].
- `chunk_block_fetch_size`: _u64_. The maximum amount of blocks to fetch per
  unroll step. The default value is 10, however for faster chains this value may
  need to be increased. Higher values will increase the indexing speed, but can
  also cause significant spikes in RPC usage.
- `rpc_url`: _String_. The [`JSON-RPC`][ethrpc] url for this chain.
- `index_trivial_events`: _bool_. Whether or not to fully index events that do
  not produce a counterparty action (`packet_recv`, `packet_acknowledgement`,
  `packet_timeout`, `update_client`). Not enabled by default. Beware that
  enabling this option will likely drastically increase this plugin's RPC usage.
- `max_cache_size`: _u32_. The maximum cache size for the `alloy`
  [`CachingLayer`]. The default value is 0, meaning no caching is performed by
  default. Different workloads may perform better with different values, and as
  such it is recommended to experiment with this value for each configuration.

## Metrics

This plugin does not yet export any metrics.

TODO: Export metrics

[deployments]: https://docs.union.build/protocol/deployments#ibc-solidity
[ethrpc]: https://ethereum.github.io/execution-apis/
[`cachinglayer`]: https://docs.rs/alloy/latest/alloy/providers/layers/struct.CacheLayer.html
[`eth_getlogs`]: https://ethereum.org/developers/docs/apis/json-rpc/#eth_getLogs
