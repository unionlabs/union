# Voyager Gno State Module

This module provides state querying functionality for the Union IBC stack
deployed on [Gno.land]. IBC state is read via smart contract queries to the core
contract. TODO: Figure out how to do queries

## Configuration

- `rpc_url`: _String_. The [Tendermint2] RPC url for this Gno.land chain.
- `ibc_core_realm`: _String_. The address of the deployed
  `ibc-union` contract on this chain. The contract will be checked to see if it
  exists at startup. Canonical deployments can be found [here][deployments].

### Module Info

This module only provides state for the `ibc-union` IBC specification, and the
[Gno IBC Interface].

[deployments]: https://docs.union.build/protocol/deployments#ibc-gno
[gno ibc interface]: https://docs.union.build/connect/implementations/#gno
[gno.land]: https://gno.land
[tendermint2]: https://github.com/gnolang/gno/tree/master/tm2
