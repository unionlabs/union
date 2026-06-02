# Union IBC Technical Specification

## Host Requirements

- Read/write key-value store, with keys and values *at least* 32 bytes.
- Some form of cross-contract calls

## Protocol Layers

### Clients

Clients provide verification logic for a counterparty chain. They track and verify the progression of the finality of said counterparty ("client updates") and verify state proofs to validate state inclusion (and exclusion) in that finalized state. Clients each only verify one counterparty, but there may be multiple clients tracking the same counterparty chain; in practice, however, there will only be one "canonical" client per counterparty.

### Connections

Connections are build on top of clients. A connection represents connection between two chains, where each side has a client that tracks the other. This connection defines the security layer in the IBC stack. Similar to clients, there can exist multiple connections between two chains, even using the same clients, however there will only be one "canonical" connection that is used per chain pair.

### Channels

Channels are built on top of connections, and are the "application" layer, where the actual business logic of a protocol lives. Channels are 1:many with connections, in that multiple channels can be built on top of the same connection. Packets are the envolope for opaque data that is interpreted by the protocol on either end of the channel, and they also contain metadata for the source/destination channel and timeout timestamp (NOTE: for historical reasons, packets also contain a timeout height field, which is no longer used and must always be set to zero). Acknowledgements are written on the destination and sent back to the source chain, enabling the protocol to take action after receipt.

## Types

NOTE: Any "interger-ish" types are non-zero and unsigned (either integers or c-like enums), since not all execution environments allow for differentiating between abscence and zero/default. This same general logic applies to all types defined in this document, although it may manifest in different ways, and will be explained further per type.

### Encoding

The Union IBC protocol uses a subset of ethabi for encoding and decoding all types. This uses the "params" encoding of the type - see [alloy-params-encoding] for more information on the difference between `abi.encode(T)` and `abi.encode((T))`.  

[alloy-params-encoding]: https://docs.rs/alloy/latest/alloy/dyn_abi/abi/index.html#encodedecode_params

### Ids

All IDs are non-zero unsigned 32 bit integers.

```hs
newtype ClientId = ClientId NonZero Int32
newtype ConnectionId = ConnectionId NonZero Int32
newtype ChannelId = ChannelId NonZero Int32
```

All of these IDs must be globally unique per-chain. Typically this is implemented with an auto-incrementing value, although this is not strictly required.

Canonical encoding: `ethabi(uint32)`

### Timestamp

Timestamps are a unix timestamp with nanosecond precision, represented by non-zero unsigned 64 bit integer.

```hs
newtype Timestamp = Timestamp NonZero Int64
```

Canonical encoding: `ethabi(uint64)`

### BatchHash

The batch hash is the unique identifier not only for a batch of packets, but also for an individual packet (the packet hash for a single packet is the hash of a batch of that single packet).

```hs
newtype BatchHash = BatchHash FixedBytes 32
```

A batch hash is a 32 byte value, produced by keccak hashing the ethabi encoded list of the single packet:

```
keccak(ethabi([packet]))
```

and for multiple packets:

```
keccak(ethabi([packet_1, packet_2, ..., packet_n]))
```

### Status

An enum representing the status of a client.

- Active = 1
- Expired = 2
- Frozen = 3

Canonical encoding: Not yet defined, there has not yet been a need for this type to be explicitly encoded in any structures.

### Connection

The connection record contains information pertaining to the connection between two clients.

```hs
data Connection = Connection
  { state :: ConnectionState 
  , client_id :: ClientId
  , counterparty_client_id :: ClientId
  , counterparty_connection_id :: Maybe ClientId
  }
```

Note that the counterparty_connection_id will be None (i.e. 0) iff state is `Init`.

Canonical encoding: ethabi((uint8, uint32, uint32, uint32))


### ConnectionState

An enum representing the state of the connection.

- Init = 1
- TryOpen = 2
- Open = 3

Canonical encoding: ethabi(uint8)

### Channel

The channel record contains information pertaining to a channel.

```hs
data Connection = Connection
  { state :: ChannelState
  , connection_id :: ConnectionId
  , counterparty_channel_id :: Maybe ChannelId
  , counterparty_port_id :: Bytes
  , version :: String
  }
```

Note that the counterparty_channel_id will be None (i.e. 0) iff state is `Init`.

Canonical encoding: ethabi((uint8,uint32,uint32,bytes,string))

### ChannelState

An enum representing the state of the channel.

- Init = 1
- TryOpen = 2
- Open = 3
- Closed = 4

Canonical encoding: ethabi(uint8)


### Packet

The packet record contains the opaque protocol-encoded data and relevant metadata for a packet sent over a channel.

```hs
data Connection = Connection
  { source_channel_id :: ChannelId
  , destination_channel_id :: ChannelId
  , data :: Bytes
  , timeout_height :: 0 -- see note below
  , timeout_timestamp :: Timestamp
  }
```

Canonical encoding: ethabi((uint32,uint32,bytes,uint64,uint64))

Note that the timestamp_height is a legacy field for timeout height that was deprecated shortly after the initial release, and it MUST always be zero. All ibc implementations MUST enforce this requirement.

## Commitments

The Union IBC protocol defines several commitment namespaces used in various parts of the stack.

Three constant values are defined for use in comments:

```
COMMITMENT_MAGIC = uint256(1 << 248) # 0x0100000000000000000000000000000000000000000000000000000000000000
COMMITMENT_MAGIC_ACK = uint256(1 << 249) # 0x0200000000000000000000000000000000000000000000000000000000000000
NON_MEMBERSHIP_COMMITMENT_VALUE = uint256(1) = 0x0000000000000000000000000000000000000000000000000000000000000001
```

The defined namespaces are:

```
const CLIENT_STATE = uint256(0)
const CONSENSUS_STATE = uint256(1)
const CONNECTIONS = uint256(2)
const CHANNELS = uint256(3)
const PACKETS = uint256(4)
const PACKET_ACKS = uint256(5)
const MEMBERSHIP_PROOF = uint256(6)
const NON_MEMBERSHIP_PROOF = uint256(7)
const PACKET_TIMEOUTS = uint256(8)
```

TODO: Add membership/nonmemership stores (for proof lens clients) and packet timeout store (for timeout commitments)

### ClientState

Stores the hash of the client state of the specified client.

```
key: client_id: ClientId -> keccak(ethabi(CLIENT_STATE, uint256(client_id)))
stored value = keccak(client state bytes)
```

### ClientConsensusState

Stores the hash of the consensus state ("checkpoint") of the specified client at the specified height.

```
key = client_id: ClientId -> height: Height -> keccak(ethabi(CONSENSUS_STATE, uint256(client_id), uint256(timestamp)))
value = keccak(consensus state bytes)
```

### Connection

Stores the hash of the connection identified by the given connection_id.

```
key = connection_id: ConnectionId -> keccak(ethabi(CONNECTIONS, uint256(connection_id)))
value = keccak(canonical_encoding(Connection))
```

### Channel

Stores the hash of the channel identified by the given channel_id.

```
key = channel_id: ChannelId -> keccak(ethabi(CHANNELS, uint256(channel_id)))
value = keccak(canonical_encoding(Channel))
```

### BatchPackets

Stores the commitment of a set of packets. TODO: Explain when the magic values are used

```
key = batch_hash: BatchHash -> keccak(ethabi(PACKETS, batch_hash))
value = bytes32
```

### BatchReceipts

Stores the commitment of a set of packet acknowledgements. TODO: Explain when the magic values are used

```
key = batch_hash: BatchHash -> keccak(ethabi(PACKET_ACKS, batch_hash))
value = bytes32
```

## Datagrams

See https://github.com/unionlabs/union/blob/main/lib/ibc-union-spec/src/datagram.rs for full types.

TODO: Add the msgs for proof lens client logic and timeout commitments
TODO: Explain proof logic of all datagram functionality
TODO: Define all the datagrams here and link to them in the various implementations.

### MsgCreateClient

```hs
data MsgCreateClient = MsgCreateClient
    { client_type :: ClientType
    , client_state_bytes :: Bytes
    , consensus_state_bytes :: Bytes
    }
```

Create a new client of the specified type, with the provided TOFU client & consensus state.

This MUST call into the specified client type's [`createClient`](#createclient) hook.

The initial client state commitment MUST be saved under the [`ClientState`](#clientstate) path, under the ID of the newly created client.

The initial consensus state commitment MUST be saved under the [`ClientConsensusState`](#clientconsensusstate) path, under the ID of the newly created client and the initial trusted height.

The client state and consensus states must also be saved on chain (either by the light client directly or the core module).

MUST emit `CreateClient`.

### MsgUpdateClient

```hs
data MsgUpdateClient = MsgUpdateClient
    { client_id :: ClientId
    , client_message :: Bytes
    }
```

Update the specified client with the provided clientmessage.

The client being updated MUST be active (see [`getStatus`](#getstatus)).

This MUST call into the specified client type's [`updateClient`](#updateclient) hook.

The updated client state commitment MUST be saved under the [`ClientState`](#clientstate) path, under the client that was updated.

The new consensus state commitment MUST be saved under the [`ClientConsensusState`](#clientconsensusstate) path, under the ID of the client that was updated and the new trusted height.

The client state and consensus states must also be saved on chain (either by the light client directly or the core module).

MUST emit `UpdateClient`.

### MsgConnectionOpenInit

```hs
data MsgConnectionOpenInit = MsgConnectionOpenInit
    { client_id :: ClientId
    , counterparty_client_id :: ClientId
    }
```

Start the connection handshake on this chain, connecting the specified client on this chain to the specified client on the counterparty chain.

MUST emit `ConnectionOpenInit`.

### MsgConnectionOpenTry

```hs
data MsgConnectionOpenTry = MsgConnectionOpenTry
    { client_id :: ClientId
    , counterparty_client_id :: ClientId
    , counterparty_connection_id :: ConnectionId
    , proof_init :: Bytes
    , proof_height :: u64
    }
```

after the connection open init on the counterparty chain, init the connection on this chain.

MUST emit `ConnectionOpenTry`.

### MsgConnectionOpenAck

```hs
data MsgConnectionOpenAck = MsgConnectionOpenAck
    { connection_id :: ConnectionId
    , counterparty_connection_id :: ConnectionId
    , proof_try :: Bytes
    , proof_height :: u64
    }
```

After the connection open try on the counterparty chain, acknowledge the connection opening back on this chain.

MUST emit `ConnectionOpenAck`.

### MsgConnectionOpenConfirm

```hs
data MsgConnectionOpenConfirm = MsgConnectionOpenConfirm
    { connection_id :: ConnectionId
    , proof_try :: Bytes
    , proof_height :: u64
    }
```

after the connection open ack on the counterparty chain, confirm the connection opening back on this chain.

MUST emit `ConnectionOpenConfirm`.

### MsgChannelOpenInit

start the channel handshake on this chain, over the specified connection id. this also calls "onChannelOpenInit" on the contract identified by the port id.

MUST emit `ChannelOpenInit`.

### MsgChannelOpenTry

after the channel open init on the counterparty chain, init the channel on this chain. this also calls "onChannelOpenTry" on the contract identified by the port id.

MUST emit `ChannelOpenTry`.

### MsgChannelOpenAck

after the channel open try on the counterparty chain, acknowledge the channel opening back on this chain. this also calls "onChannelOpenAck" on the contract identified by the stored port id.

MUST emit `ChannelOpenAck`.

### MsgChannelOpenConfirm

after the channel open ack on the counterparty chain, confirm the channel opening back on this chain. this also calls "onChannelOpenConfirm" on the contract identified by the stored port id.

MUST emit `ChannelOpenConfirm`.

### MsgPacketRecv

recv a packet sent over the specified channel to this chain. this calls "onRecvPacket" on the stored contract.

MUST emit both RecvPacket AND WriteAcknowledgement. # REVIEW: Do we support async acknowledgements?

### MsgIntentPacketRecv

intent recv a packet sent over the specified channel to this chain. this calls "onIntentRecvPacket" on the stored contract. if the protocol does not support intents, then this sub call MUST fail and the packet MUST NOT be marked as received or failed.

REVIEW: Does this emit events?

### MsgPacketAcknowledgement

receive a packet ack of a packet sent from this chain over the specified channel. this calls "onAcknowledgementPacket" on the stored contract.

MUST emit PacketAcknowledgement.

### MsgPacketTimeout

timeout a packet that was sent from this chain over the specified channel. this calls "onTimeoutPacket" on the stored contract.

MUST emit PacketTimeout.

## Events

Events are an on-chain observable action or output that is caused by a datagram.

See https://github.com/unionlabs/union/blob/main/lib/ibc-union-spec/src/event.rs for full types.

TODO: Define all the events here and link to them in the various implementations.

## Light Client Interface

Depending on the host implementation, the exact logic for core<->client communcation may differ, however light clients MUST provide the following basic functionality in some form:

### createClient

```hs
updateClient :: (ClientId, ClientState, ConsensusState) -> (ClientState, ConsensusState, Height, ChainId)
```

Given a tofu state, create a new client with said state.

### updateClient

```hs
updateClient :: (ClientId, ClientMessage) -> (ClientState, ConsensusState, Height)
```

Update a client with the given client message.

### misbehaviour

```hs
misbehaviour :: (ClientId, ClientMessage) -> ()
```

Verify a proof of misbehaviour on the counterparty chain, and freeze the client if the misbehaviour is valid. 

### verifyMembership

```hs
verifyMembership :: (ClientId, Height, Proof, Path, Value) -> Bool
```

Given a membership proof, a height, a key, and a value, verify the key and value against the stored client state an consensus state at that height.

### verifyNonMembership

```hs
verifyNonMembership :: (ClientId, Height, Proof, Path) -> Bool
```

This functions the same as verify membership, except for verifying that a key contains no value (or a zero value) rather than a specific expected value.

Note that not all chains support this functionality, and as such this is an optional feature. If the counterparty chain does not support any kind of non-membership or exclusion proof, then timeouts must be handled via a non-membership proof commitment on the counterparty chain, which is then verified via a membership proof back on the tracking chain.

### getStatus

```hs
getStatus :: ClientId -> Status
```

Return the status of the client.

### getLatestHeight

```hs
getLatestHeight :: ClientId -> Timestamp
```

Return the latest trusted height of the client.

### getTimestamp

```hs
getTimestamp :: (ClientId, Height) -> Timestamp
```

Return the timestamp of a saved consensus state at a height.

## App Interface

Depending on the host implementation, the exact logic for core<->app communication may differ, however apps MUST provide the folowing basic functionality:

### onChannelOpenInit

The hook called on channel open init.

### onChannelOpenTry

The hook called on channel open try.

### onChannelOpenAck

The hook called on channel open ack.

### onChannelOpenConfirm

The hook called on channel open confirm.

### onRecvPacket

Receive a packet, i.e. process a message sent from the counterparty of this channel.

### onIntentRecvPacket

Intent receive a packet, if supported. This is similar to onRecvPacket, except that it happens before finality of the counterparty, so certain precautions must be taken to ensure the protocol is still sound if the counterparty reorgs.

### onAcknowledgePacket

The hook called upon receipt of an acknowledgement of a packet sent from this chain to the counterparty of this channel.

### onTimeoutPacket

The hook called upon receipt of a timeout of a packet sent from this chain to the counterparty. If a timeout is received, it means that the packet did not make it to the counterparty before the timeout timestamp specified in the packet metadata.
