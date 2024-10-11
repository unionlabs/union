# Concepts

## Client Misbehaviour

IBC clients must freeze when the counterparty chain becomes byzantine and
takes actions that could fool the light client into accepting invalid state
transitions. Thus, relayers are able to submit Misbehaviour proofs that prove
that a counterparty chain has signed two Headers for the same height. This
constitutes misbehaviour as the IBC client could have accepted either header
as valid. Upon verifying the misbehaviour the IBC client must freeze at that
height so that any proof verifications for the frozen height or later fail.

Note, there is a difference between the chain-level Misbehaviour that IBC is
concerned with and the validator-level Evidence that Tendermint is concerned
with. Tendermint must be able to detect, submit, and punish any evidence of
individual validators breaking the Tendermint consensus protocol and attempting
to mount an attack. IBC clients must only act when an attack is successful
and the chain has successfully forked. In this case, valid Headers submitted
to the IBC client can no longer be trusted and the client must freeze.

Governance may then choose to override a frozen client and provide the correct,
canonical Header so that the client can continue operating after the Misbehaviour
submission.

## Connection Handshake

The connection handshake occurs in 4 steps as defined in [ICS 03](https://github.com/cosmos/ibc/blob/master/spec/core/ics-003-connection-semantics).

`ConnOpenInit` is the first attempt to initialize a connection on the executing chain.
The handshake is expected to succeed if the version selected is supported. The connection
identifier for the counterparty connection must be left empty indicating that the counterparty
must select its own identifier. The connection identifier is auto derived in the format:
`connection{N}` where N is the next sequence to be used. The counter begins at 0 and increments
by 1. The connection is set and stored in the INIT state upon success.

`ConnOpenTry` is a response to a chain executing `ConnOpenInit`. The executing chain will validate
the chain level parameters the counterparty has stored such as its chainID. The executing chain
will also verify that if a previous connection exists for the specified connection identifier
that all the parameters match and its previous state was in INIT. This may occur when both
chains execute `ConnOpenInit` simultaneously. If the connection does not exist then a connection
identifier is generated in the same format done in `ConnOpenInit`. The executing chain will verify
that the counterparty created a connection in INIT state. The executing chain will also verify
The `ClientState` and `ConsensusState` the counterparty stores for the executing chain. The
executing chain will select a version from the intersection of its supported versions and the
versions set by the counterparty. The connection is set and stored in the TRYOPEN state upon
success.

`ConnOpenAck` may be called on a chain when the counterparty connection has entered TRYOPEN. A
previous connection on the executing chain must exist in either INIT or TRYOPEN. The executing
chain will verify the version the counterparty selected. If the counterparty selected its own
connection identifier, it will be validated in the basic validation of a `MsgConnOpenAck`.
The counterparty connection state is verified along with the `ClientState` and `ConsensusState`
stored for the executing chain. The connection is set and stored in the OPEN state upon success.

`ConnOpenConfirm` is a response to a chain executing `ConnOpenAck`. The executing chain's connection
must be in TRYOPEN. The counterparty connection state is verified to be in the OPEN state. The
connection is set and stored in the OPEN state upon success.

## Connection Version Negotiation

During the handshake procedure for connections a version is agreed
upon between the two parties. This occurs during the first 3 steps of the
handshake.

During `ConnOpenInit`, party A is expected to set all the versions they wish
to support within their connection state. It is expected that this set of
versions is from most preferred to least preferred. This is not a strict
requirement for the SDK implementation of IBC because the party calling
`ConnOpenTry` will greedily select the latest version it supports that the
counterparty supports as well. A specific version can optionally be passed
as `Version` to ensure that the handshake will either complete with that
version or fail.

During `ConnOpenTry`, party B will select a version from the counterparty's
supported versions. Priority will be placed on the latest supported version.
If a matching version cannot be found an error is returned.

During `ConnOpenAck`, party A will verify that they can support the version
party B selected. If they do not support the selected version an error is
returned. After this step, the connection version is considered agreed upon.

A `Version` is defined as follows:

```go
type Version struct {
  // unique version identifier
  Identifier string
  // list of features compatible with the specified identifier
  Features []string
}
```

A version must contain a non empty identifier. Empty feature sets are allowed, but each
feature must be a non empty string.

:::warning
A set of versions should not contain two versions with the same
identifier, but differing feature sets. This will result in undefined behavior
with regards to version selection in `ConnOpenTry`. Each version in a set of
versions should have a unique version identifier.
:::

## Channel Version Negotiation

During the channel handshake procedure a version must be agreed upon between
the two parties. The selection process is largely left to the callers and
the verification of valid versioning must be handled by application developers
in the channel handshake callbacks.

During `ChanOpenInit`, a version string is passed in and set in party A's
channel state.

During `ChanOpenTry`, a version string for party A and for party B are passed
in. The party A version string must match the version string used in
`ChanOpenInit` otherwise channel state verification will fail. The party B
version string could be anything (even different than the proposed one by
party A). However, the proposed version by party B is expected to be fully
supported by party A.

During the `ChanOpenAck` callback, the application module is expected to verify
the version proposed by party B using the `MsgChanOpenAck` `CounterpartyVersion`
field. The application module should throw an error if the version string is
not valid.

In general empty version strings are to be considered valid options for an
application module.

Application modules may implement their own versioning system, such as semantic
versioning, or they may lean upon the versioning system used for in connection
version negotiation. To use the connection version semantics the application
would simply pass the proto encoded version into each of the handshake calls
and decode the version string into a `Version` instance to do version verification
in the handshake callbacks.

Implementations which do not feel they would benefit from versioning can do
basic string matching using a single compatible version.

## Sending, Receiving, Acknowledging Packets

Terminology:
**Packet Commitment** A hash of the packet stored on the sending chain.
**Packet Receipt** A single bit indicating that a packet has been received.
Used for timeouts.
**Acknowledgement** Data written to indicate the result of receiving a packet.
Typically conveying either success or failure of the receive.

A packet may be associated with one of the following states:

- the packet does not exist (ie it has not been sent)
- the packet has been sent but not received (the packet commitment exists on the
  sending chain, but no receipt exists on the receiving chain)
- the packet has been received but not acknowledged (packet commitment exists
  on the sending chain, a receipt exists on the receiving chain, but no acknowledgement
  exists on the receiving chain)
- the packet has been acknowledgement but the acknowledgement has not been relayed
  (the packet commitment exists on the sending chain, the receipt and acknowledgement
  exist on the receiving chain)
- the packet has completed its life cycle (the packet commitment does not exist on
  the sending chain, but a receipt and acknowledgement exist on the receiving chain)

Sending of a packet is initiated by a call to the `ChannelKeeper.SendPacket`
function by an application module. Packets being sent will be verified for
correctness (core logic only). If the packet is valid, a hash of the packet
will be stored as a packet commitment using the packet sequence in the key.
Packet commitments are stored on the sending chain.

A message should be sent to the receving chain indicating that the packet
has been committed on the sending chain and should be received on the
receiving chain. The light client on the receiving chain, which verifies
the sending chain's state, should be updated to the lastest sending chain
state if possible. The verification will fail if the latest state of the
light client does not include the packet commitment. The receiving chain
is responsible for verifying that the counterparty set the hash of the
packet. If verification of the packet to be received is successful, the
receiving chain should store a receipt of the packet and call application
logic if necessary. An acknowledgement may be processed and stored at this time (synchronously)
or at another point in the future (asynchronously).

Acknowledgements written on the receiving chain may be verified on the
sending chain. If the sending chain successfully verifies the acknowledgement
then it may delete the packet commitment stored at that sequence. There is
no requirement for acknowledgements to be written. Only the hash of the
acknowledgement is stored on the chain. Application logic may be executed
in conjunction with verifying an acknowledgement. For example, in fungible
cross-chain token transfer, a failed acknowledgement results in locked or
burned funds being refunded.

Relayers are responsible for reconstructing packets between the sending,
receiving, and acknowledging of packets.

IBC applications sending and receiving packets are expected to appropriately
handle data contained within a packet. For example, cross-chain token
transfers will unmarshal the data into proto definitions representing
a token transfer.

Future optimizations may allow for storage cleanup. Stored packet
commitments could be removed from channels which do not write
packet acknowledgements and acknowledgements could be removed
when a packet has completed its life cycle.

on channel closure will additionally verify that the counterparty channel has
been closed. A successful timeout may execute application logic as appropriate.

Both the packet's timeout timestamp and the timeout height must have been
surpassed on the receiving chain for a timeout to be valid. A timeout timestamp
or timeout height with a 0 value indicates the timeout field may be ignored.
Each packet is required to have at least one valid timeout field.
