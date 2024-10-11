---
order: 5
---

# State Sync

## Channels

State sync has four distinct channels. The channel identifiers are listed below.

| Name              | Number |
|-------------------|--------|
| SnapshotChannel   | 96     |
| ChunkChannel      | 97     |
| LightBlockChannel | 98     |
| ParamsChannel     | 99     |

## Message Types

### SnapshotRequest

When a new node begin state syncing, it will ask all peers it encounters if it has any
available snapshots:

| Name     | Type   | Description | Field Number |
|----------|--------|-------------|--------------|

### SnapShotResponse

The receiver will query the local ABCI application via `ListSnapshots`, and send a message
containing snapshot metadata (limited to 4 MB) for each of the 10 most recent snapshots: and stored at the application layer. When a peer is starting it will request snapshots.  

| Name     | Type   | Description                                               | Field Number |
|----------|--------|-----------------------------------------------------------|--------------|
| height   | uint64 | Height at which the snapshot was taken                    | 1            |
| format   | uint32 | Format of the snapshot.                                   | 2            |
| chunks   | uint32 | How many chunks make up the snapshot                      | 3            |
| hash     | bytes  | Arbitrary snapshot hash                                   | 4            |
| metadata | bytes  | Arbitrary application data. **May be non-deterministic.** | 5            |

### ChunkRequest

The node running state sync will offer these snapshots to the local ABCI application via
`OfferSnapshot` ABCI calls, and keep track of which peers contain which snapshots. Once a snapshot
is accepted, the state syncer will request snapshot chunks from appropriate peers:

| Name   | Type   | Description                                                 | Field Number |
|--------|--------|-------------------------------------------------------------|--------------|
| height | uint64 | Height at which the chunk was created                       | 1            |
| format | uint32 | Format chosen for the chunk.  **May be non-deterministic.** | 2            |
| index  | uint32 | Index of the chunk within the snapshot.                     | 3            |

### ChunkResponse

The receiver will load the requested chunk from its local application via `LoadSnapshotChunk`,
and respond with it (limited to 16 MB):

| Name    | Type   | Description                                                 | Field Number |
|---------|--------|-------------------------------------------------------------|--------------|
| height  | uint64 | Height at which the chunk was created                       | 1            |
| format  | uint32 | Format chosen for the chunk.  **May be non-deterministic.** | 2            |
| index   | uint32 | Index of the chunk within the snapshot.                     | 3            |
| hash    | bytes  | Arbitrary snapshot hash                                     | 4            |
| missing | bool   | Arbitrary application data. **May be non-deterministic.**   | 5            |

Here, `Missing` is used to signify that the chunk was not found on the peer, since an empty
chunk is a valid (although unlikely) response.

The returned chunk is given to the ABCI application via `ApplySnapshotChunk` until the snapshot
is restored. If a chunk response is not returned within some time, it will be re-requested,
possibly from a different peer.

The ABCI application is able to request peer bans and chunk refetching as part of the ABCI protocol.

### LightBlockRequest

To verify state and to provide state relevant information for consensus, the node will ask peers for
light blocks at specified heights.

| Name     | Type   | Description                | Field Number |
|----------|--------|----------------------------|--------------|
| height   | uint64 | Height of the light block  | 1            |

### LightBlockResponse

The receiver will retrieve and construct the light block from both the block and state stores. The
receiver will verify the data by comparing the hashes and store the header, commit and validator set
if necessary. The light block at the height of the snapshot will be used to verify the `AppHash`.

| Name          | Type                                                    | Description                          | Field Number |
|---------------|---------------------------------------------------------|--------------------------------------|--------------|
| light_block   | [LightBlock](../../../core/data_structures.md#lightblock)  | Light block at the height requested  | 1            |

State sync will use [light client verification](../../../light-client/verification/README.md) to verify
the light blocks.

If no state sync is in progress (i.e. during normal operation), any unsolicited response messages
are discarded.

### ParamsRequest

In order to build the state, the state provider will request the params at the height of the snapshot and use the header to verify it.

| Name     | Type   | Description                | Field Number |
|----------|--------|----------------------------|--------------|
| height   | uint64 | Height of the consensus params  | 1            |


### ParamsResponse

A receiver to the request will use the state store to fetch the consensus params at that height and return it to the sender.

| Name     | Type   | Description                     | Field Number |
|----------|--------|---------------------------------|--------------|
| height   | uint64 | Height of the consensus params  | 1            |
| consensus_params | [ConsensusParams](../../../core/data_structures.md#ConsensusParams) | Consensus params at the height requested | 2 |


### Message

Message is a [`oneof` protobuf type](https://developers.google.com/protocol-buffers/docs/proto#oneof). The `oneof` consists of eight messages.

| Name                 | Type                                       | Description                                  | Field Number |
|----------------------|--------------------------------------------|----------------------------------------------|--------------|
| snapshots_request    | [SnapshotRequest](#snapshotrequest)        | Request a recent snapshot from a peer        | 1            |
| snapshots_response   | [SnapshotResponse](#snapshotresponse)      | Respond with the most recent snapshot stored | 2            |
| chunk_request        | [ChunkRequest](#chunkrequest)              | Request chunks of the snapshot.              | 3            |
| chunk_response       | [ChunkRequest](#chunkresponse)             | Response of chunks used to recreate state.   | 4            |
| light_block_request  | [LightBlockRequest](#lightblockrequest)    | Request a light block.                       | 5            |
| light_block_response | [LightBlockResponse](#lightblockresponse)  | Respond with a light block                   | 6            |
| params_request  | [ParamsRequest](#paramsrequest)    | Request the consensus params at a height.                       | 7            |
| params_response | [ParamsResponse](#paramsresponse)  | Respond with the consensus params                   | 8            |
