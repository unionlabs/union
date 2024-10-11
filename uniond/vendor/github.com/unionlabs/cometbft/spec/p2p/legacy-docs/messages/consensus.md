---
order: 7
---

# Consensus

## Channel

Consensus has four separate channels. The channel identifiers are listed below.

| Name               | Number |
|--------------------|--------|
| StateChannel       | 32     |
| DataChannel        | 33     |
| VoteChannel        | 34     |
| VoteSetBitsChannel | 35     |

## Message Types

### Proposal

Proposal is sent when a new block is proposed. It is a suggestion of what the
next block in the blockchain should be.

| Name     | Type                                               | Description                            | Field Number |
|----------|----------------------------------------------------|----------------------------------------|--------------|
| proposal | [Proposal](../../../core/data_structures.md#proposal) | Proposed Block to come to consensus on | 1            |

### Vote

Vote is sent to vote for some block (or to inform others that a process does not vote in the
current round). Vote is defined in the
[Blockchain](https://github.com/cometbft/cometbft/blob/main/spec/core/data_structures.md#blockidd)
section and contains validator's
information (validator address and index), height and round for which the vote is sent, vote type,
blockID if process vote for some block (`nil` otherwise) and a timestamp when the vote is sent. The
message is signed by the validator private key.

| Name | Type                                       | Description               | Field Number |
|------|--------------------------------------------|---------------------------|--------------|
| vote | [Vote](../../../core/data_structures.md#vote) | Vote for a proposed Block | 1            |

### BlockPart

BlockPart is sent when gossiping a piece of the proposed block. It contains height, round
and the block part.

| Name   | Type                                       | Description                            | Field Number |
|--------|--------------------------------------------|----------------------------------------|--------------|
| height | int64                                      | Height of corresponding block.         | 1            |
| round  | int32                                      | Round of voting to finalize the block. | 2            |
| part   | [Part](../../../core/data_structures.md#part) | A part of the block.                   | 3            |

### NewRoundStep

NewRoundStep is sent for every step transition during the core consensus algorithm execution.
It is used in the gossip part of the CometBFT consensus protocol to inform peers about a current
height/round/step a process is in.

| Name                     | Type   | Description                            | Field Number |
|--------------------------|--------|----------------------------------------|--------------|
| height                   | int64  | Height of corresponding block          | 1            |
| round                    | int32  | Round of voting to finalize the block. | 2            |
| step                     | uint32 |                                        | 3            |
| seconds_since_start_time | int64  |                                        | 4            |
| last_commit_round        | int32  |                                        | 5            |

### NewValidBlock

NewValidBlock is sent when a validator observes a valid block B in some round r,
i.e., there is a Proposal for block B and 2/3+ prevotes for the block B in the round r.
It contains height and round in which valid block is observed, block parts header that describes
the valid block and is used to obtain all
block parts, and a bit array of the block parts a process currently has, so its peers can know what
parts it is missing so they can send them.
In case the block is also committed, then IsCommit flag is set to true.

| Name                  | Type                                                         | Description                            | Field Number |
|-----------------------|--------------------------------------------------------------|----------------------------------------|--------------|
| height                | int64                                                        | Height of corresponding block          | 1            |
| round                 | int32                                                        | Round of voting to finalize the block. | 2            |
| block_part_set_header | [PartSetHeader](../../../core/data_structures.md#partsetheader) |                                        | 3            |
| block_parts           | int32                                                        |                                        | 4            |
| is_commit             | bool                                                         |                                        | 5            |

### ProposalPOL

ProposalPOL is sent when a previous block is re-proposed.
It is used to inform peers in what round the process learned for this block (ProposalPOLRound),
and what prevotes for the re-proposed block the process has.

| Name               | Type     | Description                   | Field Number |
|--------------------|----------|-------------------------------|--------------|
| height             | int64    | Height of corresponding block | 1            |
| proposal_pol_round | int32    |                               | 2            |
| proposal_pol       | bitarray |                               | 3            |

### ReceivedVote

ReceivedVote is sent to indicate that a particular vote has been received. It contains height,
round, vote type and the index of the validator that is the originator of the corresponding vote.

| Name   | Type                                                             | Description                            | Field Number |
|--------|------------------------------------------------------------------|----------------------------------------|--------------|
| height | int64                                                            | Height of corresponding block          | 1            |
| round  | int32                                                            | Round of voting to finalize the block. | 2            |
| type   | [SignedMessageType](../../../core/data_structures.md#signedmsgtype) |                                        | 3            |
| index  | int32                                                            |                                        | 4            |

### VoteSetMaj23

VoteSetMaj23 is sent to indicate that a process has seen +2/3 votes for some BlockID.
It contains height, round, vote type and the BlockID.

| Name   | Type                                                             | Description                            | Field Number |
|--------|------------------------------------------------------------------|----------------------------------------|--------------|
| height | int64                                                            | Height of corresponding block          | 1            |
| round  | int32                                                            | Round of voting to finalize the block. | 2            |
| type   | [SignedMessageType](../../../core/data_structures.md#signedmsgtype) |                                        | 3            |

### VoteSetBits

VoteSetBits is sent to communicate the bit-array of votes a process has seen for a given
BlockID. It contains height, round, vote type, BlockID and a bit array of
the votes a process has.

| Name     | Type                                                             | Description                            | Field Number |
|----------|------------------------------------------------------------------|----------------------------------------|--------------|
| height   | int64                                                            | Height of corresponding block          | 1            |
| round    | int32                                                            | Round of voting to finalize the block. | 2            |
| type     | [SignedMessageType](../../../core/data_structures.md#signedmsgtype) |                                        | 3            |
| block_id | [BlockID](../../../core/data_structures.md#blockid)                 |                                        | 4            |
| votes    | BitArray                                                         | Round of voting to finalize the block. | 5            |

### Message

Message is a [`oneof` protobuf type](https://developers.google.com/protocol-buffers/docs/proto#oneof).

| Name            | Type                            | Description                            | Field Number |
|-----------------|---------------------------------|----------------------------------------|--------------|
| new_round_step  | [NewRoundStep](#newroundstep)   | Height of corresponding block          | 1            |
| new_valid_block | [NewValidBlock](#newvalidblock) | Round of voting to finalize the block. | 2            |
| proposal        | [Proposal](#proposal)           |                                        | 3            |
| proposal_pol    | [ProposalPOL](#proposalpol)     |                                        | 4            |
| block_part      | [BlockPart](#blockpart)         |                                        | 5            |
| vote            | [Vote](#vote)                   |                                        | 6            |
| received_vote   | [ReceivedVote](#receivedvote)	|                                        | 7            |
| vote_set_maj23  | [VoteSetMaj23](#votesetmaj23)   |                                        | 8            |
| vote_set_bits   | [VoteSetBits](#votesetbits)     |                                        | 9            |
