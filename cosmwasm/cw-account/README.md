# `cw-account`

This contract provides proxy account functionality. Both local admins (accounts on this chain) and remote admins (via [ucs03-zkgm]) are able to dispatch messages on behalf of this contract. Remote admins enable cross-chain account ownership and arbitrary message execution via the [ucs03-zkgm `Call`][proxyaccount] opcode.

## Instantiation

This contract supports both standard contract instantiation and [`frissitheto`][frissitheto].

## Entrypoints

- `set_zkgm`

  Sets the zkgm address. This is mostly intended to be used for configuring a `cw-account` init'd with a local config, however it also allows for overwriting the zkgm address if already configured.

- `add_admin`

  Adds an admin, either remote or local.

- `remove_admin`

  Removes an admin, either remote or local.

  An admin is able to remove itself, however a contract must have at least one admin at any given time.

- `dispatch`

  Dispatch a list of [`CosmosMsg`][cosmosmsg] as the proxy account.

- `on_zkgm`

  This contract implements [`Zkgmable`][zkgmable] to enable the cross-chain execution. Intents (`on_intent_zkgm`) are not supported.

  The admin is extracted from the `sender`, `destination_channel_id`, and `path` from the `OnZkgm` message.

  The provided message is expected to be a JSON-serialized list of [`CosmosMsg`][cosmosmsg], which will be executed in the order provided (the same behaviour as `dispatch`).

[cosmosmsg]: https://docs.rs/cosmwasm-std/2.2.0/cosmwasm_std/enum.CosmosMsg.html
[frissitheto]: ../../lib/frissitheto
[proxyaccount]: https://docs.union.build/ucs/03/#proxyaccount
[ucs03-zkgm]: https://docs.union.build/ucs/03
[zkgmable]: ../../lib/ucs03-zkgmable
