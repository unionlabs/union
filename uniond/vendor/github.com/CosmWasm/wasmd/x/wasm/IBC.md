# IBC specification

This documents how CosmWasm contracts are expected to interact with IBC.

## General Concepts

**IBC Enabled** - when instantiating a contract, we detect if it supports IBC messages.
  We require "feature flags" in the contract/vm handshake to ensure compatibility
  for features like staking or chain-specific extensions. IBC functionality will require
  another "feature flag", and the list of "enabled features" can be returned to the `x/wasm`
  module to control conditional IBC behavior.
  
  If this feature is enabled, it is considered "IBC Enabled", and that info will
  be stored in the ContractInfo. (For mock, we assume all contracts are IBC enabled)
  
Also, please read the [IBC Docs](https://docs.cosmos.network/master/ibc/overview.html)
for detailed descriptions of the terms *Port*, *Client*, *Connection*,
and *Channel*
  
## Overview

We use "One Port per Contract", which is the most straight-forward mapping, treating each contract 
like a module. It does lead to very long portIDs however. Pay special attention to both the Channel establishment 
(which should be compatible with standard ICS20 modules without changes on their part), as well
as how contracts can properly identify their counterparty.

(We considered on port for the `x/wasm` module and multiplexing on it, but [dismissed that idea](#rejected-ideas))

* Upon `Instantiate`, if a contract is *IBC Enabled*, we dynamically 
  bind a port for this contract. The port name is `wasm.<contract address>`,
  eg. `wasm.cosmos1hmdudppzceg27qsuq707tjg8rkgj7g5hnvnw29`
* If a *Channel* is being established with a registered `wasm.xyz` port,
  the `x/wasm.Keeper` will handle this and call into the appropriate
  contract to determine supported protocol versions during the
  [`ChanOpenTry` and `ChanOpenAck` phases](https://docs.cosmos.network/master/ibc/overview.html#channels).
  (See [Channel Handshake Version Negotiation](https://docs.cosmos.network/master/ibc/custom.html#channel-handshake-version-negotiation))
* Both the *Port* and the *Channel* are fully owned by one contract.
* `x/wasm` will allow both *ORDERED* and *UNORDERED* channels and pass that mode
  down to the contract in `OnChanOpenTry`, so the contract can decide if it accepts
  the mode. We will recommend the contract developers stick with *ORDERED* channels
  for custom protocols unless they can reason about async packet timing.
* When sending a packet, the CosmWasm contract must specify the local *ChannelID*.
  As there is a unique *PortID* per contract, that is filled in by `x/wasm`
  to produce the globally unique `(PortID, ChannelID)`
* When receiving a Packet (or Ack or Timeout), the contracts receives the local
  *ChannelID* it came from, as well as the packet that was sent by the counterparty.
* When receiving an Ack or Timeout packet, the contract also receives the
  original packet that it sent earlier.
* We do not support multihop packets in this model (they are rejected by `x/wasm`).
  They are currently not fully specified nor implemented in IBC 1.0, so let us
  simplify our model until this is well established

## Workflow

Establishing *Clients* and *Connections* is out of the scope of this
module and must be created by the same means as for `ibc-transfer`
(via the [go cli](https://github.com/cosmos/relayer) or better [ts-relayer](https://github.com/confio/ts-relayer)).
`x/wasm` will bind a unique *Port* for each "IBC Enabled" contract.

For mocks, all the Packet Handling and Channel Lifecycle Hooks are routed 
to some Golang stub handler, but containing the contract address, so we
can perform contract-specific actions for each packet. In a real setting,
we route to the contract that owns the port/channel and call one of it's various
entry points.

Please refer to the CosmWasm repo for all 
[details on the  IBC API from the point of view of a CosmWasm contract](https://github.com/CosmWasm/cosmwasm/blob/main/IBC.md).

## Future Ideas

Here are some ideas we may add in the future

### Dynamic Ports and Channels

* multiple ports per contract
* elastic ports that can be assigned to different contracts
* transfer of channels to another contract

This is inspired by the Agoric design, but also adds considerable complexity to both the `x/wasm`
implementation as well as the correctness reasoning of any given contract. This will not be
available in the first version of our "IBC Enabled contracts", but we can consider it for later,
if there are concrete user cases that would significantly benefit from this added complexity. 

### Add multihop support

Once the ICS and IBC specs fully establish how multihop packets work, we should add support for that.
Both on setting up the routes with OpenChannel, as well as acting as an intermediate relayer (if that is possible)

## Rejected Ideas
  
### One Port per Module

We decided on "one port per contract", especially after the IBC team raised
the max length on port names to allow `wasm-<bech32 address>` to be a valid port.
Here are the arguments for "one port for x/wasm" vs "one port per contract". Here 
was an alternate proposal:

In this approach, the `x/wasm` module just binds one port to handle all
modules. This can be well defined name like `wasm`. Since we always
have `(ChannelID, PortID)` for routing messages, we can reuse one port
for all contracts as long as we have a clear way to map the `ChannelID`
to a specific contract when it is being established.


* On genesis we bind the port `wasm` for all communication with the `x/wasm`
  module.
* The *Port* is fully owned by `x/wasm`
* Each *Channel* is fully owned by one contract.
* `x/wasm` only accepts *ORDERED Channels* for simplicity of contract
  correctness.

To clarify:

* When a *Channel* is being established with port `wasm`, the
  `x/wasm.Keeper` must be able to identify for which contract this
  is destined. **how to do so**??
  * One idea: the channel name must be the contract address. This means
    (`wasm`, `cosmos13d...`) will map to the given contract in the wasm module.
    The problem with this is that if two contracts from chainA want to
    connect to the same contracts on chainB, they will want to claim the
    same *ChannelID* and *PortID*. Not sure how to differentiate multiple
    parties in this way.
  * Other ideas: have a special field we send on `OnChanOpenInit` that
    specifies the destination contract, and allow any *ChannelID*.
    However, looking at [`OnChanOpenInit` function signature](https://docs.cosmos.network/master/ibc/custom.html#implement-ibcmodule-interface-and-callbacks),
    I don't see a place to put this extra info, without abusing the version field,
    which is a [specified field](https://docs.cosmos.network/master/ibc/custom.html#channel-handshake-version-negotiation):
    ```
    Versions must be strings but can implement any versioning structure. 
    If your application plans to have linear releases then semantic versioning is recommended.
    ... 
    Valid version selection includes selecting a compatible version identifier with a subset 
    of features supported by your application for that version.
    ...    
    ICS20 currently implements basic string matching with a
    single supported version.
    ```