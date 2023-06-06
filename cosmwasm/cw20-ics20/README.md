# CW20 ICS20

This is an _IBC Enabled_ contract that allows us to send CW20 tokens from one chain over the standard ICS20
protocol to the bank module of another chain. In short, it lets us send our custom CW20 tokens with IBC and use
them just like native tokens on other chains.

It is only designed to send tokens and redeem previously sent tokens. It will not mint tokens belonging
to assets originating on the foreign chain. This is different than the Golang `ibctransfer` module, but
we properly implement ICS20 and respond with an error message... let's hope the Go side handles this correctly.

## Workflow

The contract starts with minimal state. It just stores a default timeout in seconds for all packets it sends.
Most importantly it binds a local IBC port to enable channel connections.

An external party first needs to make one or more channels using this contract as one endpoint. It will use standard ics20
unordered channels for the version negotiation. Once established, it manages a list of known channels. You can use
[ts-relayer](https://github.com/confio/ts-relayer) `ibc-setup ics20` command to create these.

After there is at least one channel, you can send any CW20 token to this contract via the
[receiver pattern](https://github.com/CosmWasm/cw-plus/blob/master/packages/cw20/README.md#receiver).
The receive message must contain the channel to send over and the remote address to send to. It may optionally
include a custom timeout.

## Messages

It only accepts CW20ReceiveMsg from a cw20 contract. The data sent along with that message must be a JSON-serialized
TransferMsg:

```rust
pub struct TransferMsg {
    /// The local channel to send the packets on
    pub channel: String,
    /// The remote address to send to
    /// Don't use HumanAddress as this will likely have a different Bech32 prefix than we use
    /// and cannot be validated locally
    pub remote_address: String,
    /// How long the packet lives in seconds. If not specified, use default_timeout
    pub timeout: Option<u64>,
}
```

In addition, it supports directly sending native tokens via `ExecuteMsg::Transfer(TransferMsg)`.
You must send _exactly one_ coin denom along with the transfer message, and that amount will be transferred
to the remote host.

## Queries

Queries only make sense relative to the established channels of this contract.

- `Port{}` - returns the port ID this contract has bound, so you can create channels. This info can be queried
  via wasmd contract info query, but we expose another query here for convenience.
- `ListChannels{}` - returns a (currently unpaginated) list of all channels that have been created on this contract.
  Returns their local channelId along with some basic metadata, like the remote port/channel and the connection they
  run on top of.
- `Channel{id}` - returns more detailed information on one specific channel. In addition to the information available
  in the list view, it returns the current outstanding balance on that channel, as well as the total amount that
  has ever been sent on the channel.

## IBC Responses

These are defined by the ICS20 spec.

Notably, each Channel has a balance of tokens sent over that channel. If an incoming transfer request comes in for
a denom it does not know, or for a balance larger than we have sent, we will return an error in the acknowledgement
packet.
